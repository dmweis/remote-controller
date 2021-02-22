
function mountJoystick(element, color, joystickData, deadZone) {
    return nipplejs.create({ zone: element, color: color }).on('added',
        function (evt, nipple) {
            nipple.on('move',
                function (evt, arg) {
                    const size = arg.distance / 50;
                    if (size < deadZone) {
                        joystickData.x = 0;
                        joystickData.y = 0;
                        return;
                    }
                    joystickData.x = Math.sin(arg.angle.radian) * size;
                    joystickData.y = -Math.cos(arg.angle.radian) * size;
                });
            nipple.on('start',
                function () {
                    joystickData.x = 0;
                    joystickData.y = 0;
                });
            nipple.on('end',
                function () {
                    joystickData.x = 0;
                    joystickData.y = 0;
                });
        });
}

function deepCopy(object) {
    return JSON.parse(JSON.stringify(object));
}

function axisToCombined(move, rotation) {
    return {
        lx: move.x,
        ly: move.y,
        rx: rotation.x,
        ry: rotation.y
    };
}

function axisEquals(left, right) {
    return left.lx == right.lx && left.ly == right.ly && left.rx == right.rx && left.ry == right.ry;
}

function mountTouchScreenControls(connection, deadzone) {

    var moveJoystickData = { x: 0, y: 0 };
    var rotationJoystickData = { x: 0, y: 0 };

    var moveJoystickManager = mountJoystick(document.getElementById("move_joystick_zone"), "red", moveJoystickData, deadzone);
    var rotationJoystickManager = mountJoystick(document.getElementById("rotate_joystick_zone"), "navy", rotationJoystickData, deadzone)

    var lastState = axisToCombined(moveJoystickData, rotationJoystickData);

    function touchJoystickListener() {
        const current = axisToCombined(moveJoystickData, rotationJoystickData);
        if (!axisEquals(current, lastState)) {
            lastState = deepCopy(current);
            connection.send(current);
        }
    };
    var touchJoystickInterval = setInterval(touchJoystickListener, 50);

    return {
        detach: function () {
            moveJoystickManager.destroy();
            rotationJoystickManager.destroy();
            clearInterval(touchJoystickInterval);
        }
    };
}
