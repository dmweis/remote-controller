console.log("Hello world");
var conn = null;
var gamepadTimer = null;
var touchScreenManager = null;

function connect() {
    disconnect();
    var wsUri = "ws://" + window.location.host + "/ws/";
    conn = new WebSocket(wsUri);
    console.log("Connecting...");
    conn.onopen = function () {
        console.log("Connected.");
        const debouncer = new Debouncer(conn);
        gamepadTimer = setGamepadWatchdog(debouncer, 0.2);
        touchScreenManager = mountTouchScreenControls(debouncer, 0.05);
    };
    conn.onmessage = function (e) {
        console.log("Received: " + e.data);
    };
    conn.onclose = function () {
        clearInterval(gamepadTimer);
        if (touchScreenManager) {
            touchScreenManager.detach();
        }
        gamepadTimer = null;
        touchScreenManager = null;
        console.log("Disconnected.");
        conn = null;
    };
    conn.onerror = function () {
        conn.close();
        conn = null;
    };
}
function disconnect() {
    if (conn != null) {
        console.log("Disconnecting...");
        conn.close();
        conn = null;
    }
}

function reconnectionTimer() {
    if (conn == null) {
        connect();
    }
}
var reconnectionTimer = setInterval(reconnectionTimer, 1000);
connect();

const fullscreenButton = document.getElementById("fullscreen_button");

function isFullscreen() {
    return document.fullscreenElement != null;
}

fullscreenButton.addEventListener("click", event => {
    if (isFullscreen()) {
        document.exitFullscreen()
    } else {
        document.body.requestFullscreen();
    }
});
