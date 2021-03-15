const button_container = document.getElementById("button_container");

function populateButtons(actions) {
    for (action of actions) {
        const actionDescription = action.description;
        const actionId = action.id;
        const button = document.createElement("button");
        button.classList.add("action_button");
        button.innerHTML = actionDescription;
        button.addEventListener("click", () => {
            fetch("http://" + window.location.host + "/action/", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ "id": actionId }),
            });
        });
        button_container.appendChild(button);
    }
}

fetch("http://" + window.location.host + "/actions").then((response) => {
    response.json().then((message) => {
        populateButtons(message.actions)
    });
});




