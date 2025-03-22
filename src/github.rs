use crate::{
    config::GitHubConfig,
    errors::InternalError,
    model::{event::Event, events::Events},
};
use eyre::eyre;
use jsonwebtoken::EncodingKey;
use log::{trace, warn};
use octocrab::{
    Octocrab, OctocrabBuilder,
    models::repos::{CommitAuthor, Object},
    params::repos::Reference,
    pulls::PullRequestHandler,
    repos::RepoHandler,
};
use std::{collections::HashSet, fs};
use url::Url;

/// The higher suffix number to add to a branch name.
const MAX_SUFFIX: u32 = 9;

async fn build_octocrab(config: &GitHubConfig) -> Result<Octocrab, InternalError> {
    let file_contents = fs::read(&config.private_key)?;
    let key = EncodingKey::from_rsa_pem(&file_contents)?;
    let octocrab = OctocrabBuilder::new()
        .app(config.app_id.into(), key)
        .build()?;

    // Get the installation for the repository we care about.
    let installation = octocrab
        .apps()
        .get_repository_installation(&config.owner, &config.repository)
        .await?;

    // Make an Octocrab for that installation.
    Ok(octocrab.installation(installation.id)?)
}

fn get_repo_pulls<'a>(
    octocrab: &'a Octocrab,
    config: &GitHubConfig,
) -> Result<(RepoHandler<'a>, PullRequestHandler<'a>), InternalError> {
    Ok((
        octocrab.repos(&config.owner, &config.repository),
        octocrab.pulls(&config.owner, &config.repository),
    ))
}

/// Creates a branch for the PR to add the given event, and returns its name.
async fn create_branch(
    repo: &RepoHandler<'_>,
    prefix: &str,
    event: &Event,
    head_sha: &str,
) -> Result<String, InternalError> {
    // Create the branch, retrying with different suffixes if it already exists.
    let pr_branch_base = format!(
        "{}-{}-{}-{}",
        prefix,
        to_safe_filename(&event.country),
        to_safe_filename(&event.city),
        to_safe_filename(&event.name),
    );

    let mut last_error = eyre!("Failed to create branch for event PR.");
    for suffix in 0..=MAX_SUFFIX {
        let branch_name = if suffix == 0 {
            pr_branch_base.clone()
        } else {
            format!("{}{}", pr_branch_base, suffix)
        };
        trace!("Creating branch \"{}\"", branch_name);
        if let Err(e) = repo
            .create_ref(&Reference::Branch(branch_name.clone()), head_sha)
            .await
        {
            if matches!(&e, octocrab::Error::GitHub {source, .. } if source.message == "Reference already exists")
            {
                // Probably the branch already exists, let the loop try a different suffix.
                last_error = e.into();
            } else {
                // Some other error, return immediately.
                return Err(e.into());
            }
        } else {
            return Ok(branch_name);
        }
    }

    warn!(
        "Failed to create PR branch {} after trying all suffixes: {}",
        pr_branch_base, last_error
    );
    Err(InternalError::Internal(last_error))
}

/// Creates a PR to add the given event to the given file.
///
/// Returns the URL of the new PR.
pub async fn add_event_to_file(
    event: Event,
    filename: &str,
    email: Option<&str>,
    config: &GitHubConfig,
) -> Result<Url, InternalError> {
    let octocrab = build_octocrab(config).await?;
    let (repo, pulls) = get_repo_pulls(&octocrab, config)?;

    let head_sha = sha_for_branch(&repo, &config.main_branch).await?;
    let pr_branch = create_branch(&repo, "add", &event, &head_sha).await?;

    let author = email.map(|email| CommitAuthor {
        name: "Add form user".to_string(),
        email: email.to_string(),
        date: None,
    });

    // Create a commit to add or modify the file.
    let commit_message = format!("Add {} in {}", event.name, event.city);
    if let Ok(contents) = repo
        .get_content()
        .path(filename)
        .r#ref(&pr_branch)
        .send()
        .await
    {
        // File already exists, add to it.
        let existing_file = &contents.items[0];
        trace!("Got existing file, sha {}", existing_file.sha);
        let existing_content = existing_file.decoded_content().unwrap();
        let mut events = Events::load_str(&existing_content).map_err(InternalError::Internal)?;

        // Append event to it and sort.
        events.events.push(event);
        events.sort();
        let new_content = events.to_yaml_string().map_err(InternalError::Internal)?;

        // Update the file
        let mut update = repo
            .update_file(filename, &commit_message, new_content, &existing_file.sha)
            .branch(&pr_branch);
        if let Some(author) = author {
            update = update.author(author);
        }
        let update = update.send().await?;
        trace!("Update: {:?}", update);
    } else {
        // File doesn't exist, create it.
        let new_events = Events {
            events: vec![event.clone()],
        };
        let content = new_events
            .to_yaml_string()
            .map_err(InternalError::Internal)?;
        let mut create = repo
            .create_file(filename, &commit_message, content)
            .branch(&pr_branch);
        if let Some(author) = author {
            create = create.author(author);
        }
        let create = create.send().await?;
        trace!("Create: {:?}", create);
    }

    // Create PR for the branch.
    let pr = pulls
        .create(&commit_message, &pr_branch, &config.main_branch)
        .body("Added from web form.")
        .send()
        .await?;
    trace!("Made PR {:?}", pr);
    let pr_url = pr
        .html_url
        .ok_or_else(|| InternalError::Internal(eyre!("PR missing html_url")))?;
    Ok(pr_url)
}

/// Creates a PR to edit the given event in the given file.
///
/// Returns the URL of the new PR.
pub async fn edit_event_in_file(
    filename: &str,
    original_event: &Event,
    new_event: Event,
    email: Option<&str>,
    config: &GitHubConfig,
) -> Result<Url, InternalError> {
    let octocrab = build_octocrab(config).await?;
    let (repo, pulls) = get_repo_pulls(&octocrab, config)?;

    let head_sha = sha_for_branch(&repo, &config.main_branch).await?;
    let pr_branch = create_branch(&repo, "edit", &new_event, &head_sha).await?;

    let author = email.map(|email| CommitAuthor {
        name: "Add form user".to_string(),
        email: email.to_string(),
        date: None,
    });

    // Create a commit to add or modify the file.
    let commit_message = format!("Edit {} in {}", new_event.name, new_event.city);

    // Find the existing file.
    let contents = repo
        .get_content()
        .path(filename)
        .r#ref(&pr_branch)
        .send()
        .await?;
    let existing_file = &contents.items[0];
    trace!("Got existing file, sha {}", existing_file.sha);
    let existing_content = existing_file.decoded_content().unwrap();
    let mut events = Events::load_str(&existing_content).map_err(InternalError::Internal)?;

    // Replace the original event with the new version and sort.
    let mut original_event_without_source = original_event.clone();
    original_event_without_source.source = None;
    for e in &mut events.events {
        if e == &original_event_without_source {
            *e = new_event;
            break;
        }
    }
    events.sort();
    let new_content = events.to_yaml_string().map_err(InternalError::Internal)?;

    // Update the file
    let mut update = repo
        .update_file(filename, &commit_message, new_content, &existing_file.sha)
        .branch(&pr_branch);
    if let Some(author) = author {
        update = update.author(author);
    }
    let update = update.send().await?;
    trace!("Update: {:?}", update);

    // Create PR for the branch.
    let pr = pulls
        .create(&commit_message, &pr_branch, &config.main_branch)
        .body("Added from web form.")
        .send()
        .await?;
    trace!("Made PR {:?}", pr);
    let pr_url = pr
        .html_url
        .ok_or_else(|| InternalError::Internal(eyre!("PR missing html_url")))?;
    Ok(pr_url)
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

/// Converts the given string to a suitable filename by converting it to lowercase, replacing spaces
/// with underscores, and removing special characters.
///
/// The returned string will only contain ASCII alphanumeric characters, underscores and hyphens.
pub fn to_safe_filename(s: &str) -> String {
    let mut filename = s.to_lowercase().replace(' ', "_");
    filename.retain(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    filename.truncate(30);
    filename
}

/// Value returned by [`choose_file_for_event`] when the event is a duplicate of an existing one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DuplicateEvent {
    pub existing: Event,
    pub merged: Event,
}

/// Checks whether the given event is a duplicate of any event we already know about, or what file
/// it might belong in.
#[allow(clippy::result_large_err)]
pub fn choose_file_for_event(events: &Events, event: &Event) -> Result<String, DuplicateEvent> {
    let mut organisation_files = HashSet::new();
    let mut city_files = HashSet::new();
    for existing_event in &events.events {
        if let Some(merged) = existing_event.merge(event) {
            return Err(DuplicateEvent {
                existing: existing_event.to_owned(),
                merged,
            });
        } else if let Some(source) = &existing_event.source {
            if event.organisation.is_some() && event.organisation == existing_event.organisation {
                organisation_files.insert(source.to_owned());
            }
            if event.country == existing_event.country && event.city == existing_event.city {
                city_files.insert(source.to_owned());
            }
        }
    }

    let chosen_file = if !organisation_files.is_empty() {
        organisation_files.iter().next().unwrap().to_owned()
    } else if city_files.len() == 1 {
        city_files.iter().next().unwrap().to_owned()
    } else {
        format!(
            "events/{}/{}.yaml",
            to_safe_filename(&event.country),
            to_safe_filename(&event.city),
        )
    };

    trace!("Possible files for organisation: {:?}", organisation_files);
    trace!("Possible files for city: {:?}", city_files);
    trace!("Chosen file: {}", chosen_file);

    Ok(chosen_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_filenames() {
        assert_eq!(to_safe_filename("Southend-on-Sea"), "southend-on-sea");
        assert_eq!(
            to_safe_filename("weird'\"@\\/ characters"),
            "weird_characters"
        )
    }
}
