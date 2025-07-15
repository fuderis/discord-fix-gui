import "modules/base.js";
import "modules/form.js";

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
                new Node("#active-bat").set_attr("disabled");
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
