/** The first timezone in the list, which will be the default value. */
const FIRST_TIMEZONE = "Africa/Abidjan";

function add_input(list, name, type, id, datalist) {
  var input = document.createElement("input");
  input.setAttribute("type", type);
  input.setAttribute("name", name);
  input.setAttribute("id", id);
  input.setAttribute("list", datalist);
  var li = document.createElement("li");
  li.appendChild(input);
  list.appendChild(li);
}

function update_inputs() {
  if (this.children.length > 1) {
    var second_to_last_input =
      this.children[this.children.length - 2].lastElementChild;
  }
  var last_input = this.lastElementChild.lastElementChild;
  if (last_input.value != "") {
    // Last element has been entered, add a new one.
    add_input(
      this,
      last_input.name,
      last_input.type,
      last_input.id,
      last_input.list?.id
    );
    last_input.id = null;
  } else if (second_to_last_input != null && second_to_last_input.value == "") {
    // Last two elements are empty, remove one.
    second_to_last_input.id = last_input.id;
    this.lastElementChild.remove();
  }
}

function update_datetimes() {
  let with_time = document.getElementById("with_time").checked;

  Array.from(document.getElementsByClassName("times")).forEach((element) => {
    element.style.display = with_time ? "grid" : "none";
    Array.from(element.getElementsByTagName("input")).forEach((input) => {
      input.required = with_time;
    });
  });
  Array.from(document.getElementsByClassName("dates")).forEach((element) => {
    element.style.display = with_time ? "none" : "grid";
    Array.from(element.getElementsByTagName("input")).forEach((input) => {
      input.required = !with_time;
    });
  });
}

function update_timezone() {
  let country = document.getElementById("country").value;
  let state = document.getElementById("state").value;
  let country_state = country + "/" + state;
  let timezone_field = document.getElementById("timezone");

  if (
    timezone.value == FIRST_TIMEZONE &&
    DEFAULT_TIMEZONES.has(country_state)
  ) {
    timezone_field.value = DEFAULT_TIMEZONES.get(country_state);
  }
}

function initialise() {
  document.getElementById("links_list").oninput = update_inputs;
  document.getElementById("bands_list").oninput = update_inputs;
  document.getElementById("callers_list").oninput = update_inputs;
  document.getElementById("with_time").onchange = update_datetimes;
  document.getElementById("country").onchange = update_timezone;
  document.getElementById("state").onchange = update_timezone;

  update_datetimes();
  update_timezone();
}

window.onload = initialise;
