function setGamepadWatchdog(websocketConnection, deadzone) {
    function axisEquals(left, right) {
        return left.lx === right.lx &&
            left.ly === right.ly &&
            left.rx === right.rx &&
            left.ry === right.ry;
    };
    function applyDeadzone(gamepadData) {
        if (Math.abs(gamepadData.lx) < deadzone) gamepadData.lx = 0;
        if (Math.abs(gamepadData.ly) < deadzone) gamepadData.ly = 0;
        if (Math.abs(gamepadData.rx) < deadzone) gamepadData.rx = 0;
        if (Math.abs(gamepadData.ry) < deadzone) gamepadData.ry = 0;
    };
    let lastGamepadData = {
        lx: 0,
        ly: 0,
        rx: 0,
        ry: 0,
        id: 0,
    };
    let state = {
        fullscreen: false,
        fullscreenButtonDown: false,
    };
    function processGamepadData(gamepadData) {
        applyDeadzone(gamepadData);
        if (!axisEquals(lastGamepadData, gamepadData)) {
            lastGamepadData = gamepadData;
            // send data
            gamepadMessage = {
                lx: -gamepadData.lx,
                ly: -gamepadData.ly,
                rx: -gamepadData.rx,
                ry: -gamepadData.ry
            };
            websocketConnection.send(JSON.stringify(gamepadMessage));
        }
        if (!state.fullscreenButtonDown && gamepadData.buttons[9].pressed && !state.fullscreen) {
            document.body.requestFullscreen();
            state.fullscreen = true;
            state.fullscreenButtonDown = true;
        } else if (!state.fullscreenButtonDown && gamepadData.buttons[9].pressed && state.fullscreen) {
            document.exitFullscreen()
            state.fullscreen = false;
            state.fullscreenButtonDown = true;
        } else if (!gamepadData.buttons[9].pressed) {
            state.fullscreenButtonDown = false;
        }
    };

    function queryGamepads() {
        var gamepads = navigator.getGamepads();
        for (let index = 0; index < gamepads.length; index++) {
            const gamepad = gamepads[index];
            if (gamepad) {
                let gamepadData = {
                    lx: gamepad.axes[1],
                    ly: gamepad.axes[0],
                    rx: gamepad.axes[3],
                    ry: gamepad.axes[2],
                    id: gamepad.index,
                    buttons: gamepad.buttons,
                };
                processGamepadData(gamepadData);
                return;
            }
        }
    };

    return setInterval(queryGamepads, 50);
}