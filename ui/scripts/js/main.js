// The DOM element controller
class Node {
    elem = undefined;

    constructor(elem) {
        // html tag:
        if (typeof elem === "string" && elem.trim().startsWith("<")) {
            let template = document.createElement("template");
            template.innerHTML = elem.trim();

            this.elem = template.content.firstElementChild;
            if (!this.elem) throw new Error("Failed to create a new element from HTML");
        }
        // selector:
        else if (typeof elem === "string") {
            this.elem = document.querySelector(elem);
            if (!this.elem) throw new Error("Element not found");
        }
        // element:
        else if (elem instanceof Element) {
            this.elem = elem;
        }
        // error:
        else {
            throw new Error("Element constructor expects a selector string or a DOM element");
        }
    }

    // Get/set attribute value
    attr(name, value) {
        if (value === undefined) {
            return this.elem.getAttribute(name);
        } else {
            this.elem.setAttribute(name, value)
        }
    }

    // Get/set ID
    id(name) {
        if (name === undefined) {
            return this.elem.getAttribute("id");
        } else {
            this.elem.setAttribute("id", name)
        }
    }

    // Get/set class
    class(name) {
        if (name === undefined) {
            return this.elem.classList;
        } else {
            this.elem.classList.add(name);
        }
    }

    // Get/set text contents value
    text(value) {
        if (value === undefined) {
            return this.elem.textContent;
        } else {
            this.elem.textContent = value;
        }
    }

    // Get/set inner html value
    html(value) {
        if (value === undefined) {
            return this.elem.innerHTML;
        } else {
            this.elem.innerHTML = value;
        }
    }

    // Get/set outer html value
    outer_html(value) {
        if (value === undefined) {
            return this.elem.outerHTML;
        } else {
            this.elem.outerHTML = value;
        }
    }

    // Set event handler
    event(name, handler) {
        this.elem.addEventListener(name, handler);
    }

    // Insert node
    insert(index, node) {
        let elem = node instanceof Node ? node.elem : node;
        if (!(elem instanceof Element)) {
            throw new Error("The node must be an instance of a Node or a DOM element.");
        }

        // get children nodes:
        let children = Array.from(this.elem.children);
        let len = children.length;

        // gen index:
        if (index < 0) {
            index = len + index;
            if (index < 0) index = 0;
        }
        if (index > len) index = len;

        // insert elem:
        if (index === len) {
            this.elem.appendChild(elem);
        } else {
            this.elem.insertBefore(elem, children[index]);
        }
    }
}


const events = window.__TAURI__.event;
const invoke = window.__TAURI__.core.invoke;

// Form controller
class Form {
    constructor(form) {
        // selector:
        if (typeof form === "string") {
            this.form = document.querySelector(form);
            if (!this.form) throw new Error("Form not found");
        }
        // element:
        else if (form instanceof Element) {
            this.form = form;
        }
        // error:
        else {
            throw new Error("Form constructor expects a selector string or a DOM element");
        }
    }

    // Get field by name
    field(name) {
        let fields = this.form.querySelectorAll(`[name="${name}"]`);
        if (!fields.length) return undefined;

        let type = fields[0].type;

        // radio:
        if (type === "radio") {
            let checked = this.form.querySelector(`[name="${name}"]:checked`);
            return checked ? checked.value : null;
        }
        // checkbox:
        else if (type === "checkbox") {
            if (fields.length > 1) {
                return Array.from(fields)
                    .filter(f => f.checked)
                    .map(f => f.value);
            }
            return fields[0].checked;
        }
        // other:
        else {
            return fields[0].value;
        }
    }

    // Serialize form to json
    serialize() {
        let data = {};

        this.form.querySelectorAll("[name]").forEach(el => {
            let name = el.name;
            if (data.hasOwnProperty(name)) return;

            if (el.type === "checkbox") {
                if (!data[name]) data[name] = [];
                if (el.checked) data[name].push(el.value);
            }
            else if (el.type === "radio") {
                if (el.checked) {
                    data[el.name] = el.value;
                }
            }
            else {
                data[el.name] = el.value;
            }
        });

        return data;
    }
}


// Forms delegator
class Forms {
    timers = new Map();
    delay = 1000;

    constructor(delegator) {
        // selector:
        if (typeof delegator === "string") {
            this.delegator = document.querySelector(delegator);
            if (!this.delegator) throw new Error("Forms delegator not found");
        }
        // element:
        else if (delegator instanceof Element) {
            this.delegator = delegator;
        }
        // error:
        else {
            throw new Error("Forms delegator constructor expects a selector string or a DOM element");
        }
    }

    // Set 'input' event handler
    oninput(handler) {
        this.delegator.addEventListener("input", (event) => {
            let input = event.target;
            let form = new Form(input.closest("form"));
            let target_id = form.form.getAttribute("target");

            // reset form timer:
            if (this.timers.has(target_id)) {
                clearTimeout(this.timers.get(target_id));
            }

            // start form timer:
            this.timers.set(target_id, setTimeout(async () => {
                handler(form, input, target_id);
            }, this.delay));
        });
    }

    // Set 'submit' event handler
    onsubmit(handler) {
        this.delegator.addEventListener("submit", (event) => {
            event.preventDefault();
            let form = new Form(event.target);
            let target_id = form.form.getAttribute("target");

            handler(form, target_id);
        });
    }
}


document.addEventListener('DOMContentLoaded', () => {
    // Start process
    new Node('#power').event('click', () => {
        invoke("start_process", { name: "Developer" })
            .then(name => {
                new Node("#power").class("enabled");

                console.log(`The process '${name}' is started!`);
            })
            .catch(e => console.error(e))
    });
    
    // Stop process
    new Node('#power').event('click', () => {
        invoke("stop_process", { name: "Developer" })
            .then(name => {
                new Node("#power").class().remove("enabled");
                console.log(`The process '${name}' is stopped!`);
            })
            .catch(e => console.error(e))
    });
});
