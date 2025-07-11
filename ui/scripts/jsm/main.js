import "modules/base.js";
import "modules/form.js";

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
