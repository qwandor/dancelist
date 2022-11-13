use crate::{
    errors::InternalError,
    model::{event::Event, events::Events},
};
use eyre::eyre;
use log::info;
use octocrab::{
    models::repos::Object, params::repos::Reference, repos::RepoHandler, OctocrabBuilder,
};

const MAIN_BRANCH: &str = "main";
const OWNER: &str = "qwandor";
const REPO: &str = "dancelist-data";
const PAT: &str = "";

/// Creates a PR to add the given event to the given file.
pub async fn add_event_to_file(event: Event, filename: String) -> Result<(), InternalError> {
    let octocrab = OctocrabBuilder::new()
        .personal_token(PAT.to_string())
        .build()?;

    let new_events = Events {
        events: vec![event.clone()],
    };
    let yaml = serde_yaml::to_string(&new_events)?;

    // Create branch with change to file.
    let commit_message = format!("Add {} in {}", event.name, event.city);
    let pr_branch = format!(
        "add-{}-{}-{}",
        to_filename(&event.country),
        to_filename(&event.city),
        to_filename(&event.name),
    );

    info!("Creating branch \"{}\"", pr_branch);
    let repo = octocrab.repos(OWNER, REPO);
    let head_sha = sha_for_branch(&repo, MAIN_BRANCH).await?;
    // TODO: Check if branch already exists, pick a different name
    repo.create_ref(&Reference::Branch(pr_branch.clone()), head_sha)
        .await?;

    if let Ok(contents) = repo
        .get_content()
        .path(&filename)
        .r#ref(&pr_branch)
        .send()
        .await
    {
        // File already exists, add to it.
        let existing_file = &contents.items[0];
        let existing_content = existing_file.decoded_content().unwrap();

        // Append event to it.
        let formatted_event = yaml.trim_start_matches("---\nevents:\n");
        let new_content = format!("{}\n{}", existing_content, formatted_event);

        info!("Got existing file, sha {}", existing_file.sha);
        // Update the file
        let update = repo
            .update_file(&filename, &commit_message, new_content, &existing_file.sha)
            .branch(&pr_branch)
            .send()
            .await?;
        info!("Update: {:?}", update);
    } else {
        // File doesn't exist, create it.
        let content = yaml.replacen(
            "---",
            "# yaml-language-server: $schema=../../events_schema.json",
            1,
        );
        let create = repo
            .create_file(&filename, &commit_message, content)
            .branch(&pr_branch)
            .send()
            .await?;
        info!("Create: {:?}", create);
    }

    // Create PR for the branch.
    let pr = octocrab
        .pulls(OWNER, REPO)
        .create(&commit_message, &pr_branch, MAIN_BRANCH)
        .body("Added from web form.")
        .send()
        .await?;
    info!("Made PR {:?}", pr);

    Ok(())
}

/// Returns the SHA for the current head of the given branch.
async fn sha_for_branch(
    repo: &RepoHandler<'_>,
    branch_name: &str,
) -> Result<String, InternalError> {
    let head = repo
        .get_ref(&Reference::Branch(branch_name.to_owned()))
        .await?;
    if let Object::Commit { sha, .. } = head.object {
        Ok(sha)
    } else {
        Err(InternalError::Internal(eyre!(
            "Ref {} was not a commit.",
            branch_name
        )))
    }
}

/// Converts the given string to a suitable filename.
fn to_filename(s: &str) -> String {
    s.to_lowercase().replace(' ', "_")
}
