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
    // Check attribute for exists
    has_attr(name) {
        return this.elem.getAttribute(name) !== undefined;
    }
    // Set attribute
    set_attr(name, value) {
        this.elem.setAttribute(name, value)
    }
    // Remove attribute
    remove_attr(name) {
        this.elem.removeAttribute(name)
    }

    // Get/set ID
    id(name) {
        if (name === undefined) {
            return this.elem.getAttribute("id");
        } else {
            this.elem.setAttribute("id", name)
        }
    }
    // Set ID
    set_id(value) {
        this.elem.setAttribute("id", value)
    }
    // Remove ID
    remove_id() {
        this.elem.removeAttribute("id")
    }

    // Get/Check class list
    class(name) {
        if (name === undefined) {
            return this.elem.classList;
        } else {
            return this.elem.classList.contains(name);
        }
    }
    // Check class name for exists
    has_class(name) {
        return this.elem.classList.contains(name);
    }
    // Add class name
    set_class(name) {
        this.elem.classList.add(name);
    }
    // Remove class name
    remove_class(name) {
        this.elem.classList.remove(name);
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
    insert(node, index) {
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


events.listen("process-runned", ({ payload }) => {
    let power_button = new Node("#power");

    power_button.set_class("enabled");
    power_button.remove_attr("disabled");
    new Node("#active-bat").set_attr("disabled");
});

events.listen("process-stopped", ({ payload }) => {
    let power_button = new Node("#power");

    power_button.remove_class("enabled");
    power_button.remove_attr("disabled");
    new Node("#active-bat").remove_attr("disabled");
});

document.addEventListener('DOMContentLoaded', () => {
    // Get process status:
    invoke("get_status", {})
        .then(status => {
            if (status) {
                new Node("#power").set_class("enabled");
            }
        })
        .catch(e => console.error(e))
    
    // Get .bat files list:
    invoke("get_bats_list", {})
        .then(bats => {
            let container = new Node("#active-bat .container");
            
            bats.forEach(bat => {
                let bat_node = new Node(bat);
                let bat_input = bat_node.elem.querySelector("input[type=\"radio\"]");

                if (bat_input.checked) {
                    new Node("#active-bat .active span").text(bat_input.value);
                }

                container.insert(bat_node, -1);
            });
        })
        .catch(e => console.error(e))

    // Run/Stop process:
    new Node('#power').event('click', () => {
        let power_button = new Node("#power");
        
        power_button.set_attr("disabled")
        
        invoke((power_button.has_class("enabled"))? "stop_process" : "run_process", {})
            .then(_ => {})
            .catch(e => console.error(e))
    });

    // Update active .bat:
    new Node("#active-bat").event("input", (e) => {
        let batName = e.target.value;

        invoke("set_active_bat", { batName })
            .then(_ => {})
            .catch(e => console.error(e))
    });
});
