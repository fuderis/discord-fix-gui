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
