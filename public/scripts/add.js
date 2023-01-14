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

function initialise() {
  document.getElementById("links_list").oninput = update_inputs;
  document.getElementById("bands_list").oninput = update_inputs;
  document.getElementById("callers_list").oninput = update_inputs;
}

window.onload = initialise;
