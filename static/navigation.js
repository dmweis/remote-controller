const canvas = document.getElementById("navigation_canvas");
const ctx = canvas.getContext("2d");

canvas.width = canvas.offsetWidth;
canvas.height = canvas.offsetHeight;
let width = canvas.width;
let height = canvas.height;

function clear() {
  ctx.fillStyle = "green";
  ctx.fillRect(0, 0, width, height);
  ctx.fillStyle = "red";
  ctx.fillRect(10, 10, width - 20, height - 20);
}

function updateCanvasSize() {
  canvas.width = canvas.offsetWidth;
  canvas.height = canvas.offsetHeight;

  width = canvas.width;
  height = canvas.height;
  clear();
}

window.addEventListener("resize", updateCanvasSize);

updateCanvasSize();

let lastTouch = { x: 0, y: 0 };
let mouseDown = false;

function startDrawing(e) {
  clear();
  lastTouch = { x: e.offsetX, y: e.offsetY };
  mouseDown = true;
}

function mouseMoveEvents(e) {
  if (mouseDown === false) {
    return;
  }
  clear();
  ctx.beginPath();
  ctx.strokeStyle = "black";
  ctx.lineWidth = 1;
  ctx.moveTo(lastTouch.x, lastTouch.y);
  ctx.lineTo(e.offsetX, e.offsetY);
  ctx.stroke();
  ctx.closePath();
}

function endDrawing(e) {
  mouseDown = false;
  clear();
  ctx.beginPath();
  ctx.strokeStyle = "black";
  ctx.lineWidth = 1;
  ctx.moveTo(lastTouch.x, lastTouch.y);
  ctx.lineTo(e.offsetX, e.offsetY);
  ctx.stroke();
  ctx.closePath();

  // I don't know what's with that 10
  // but it's definitely off by 10 pixels... so
  const canvasTouch = {
    width: width + 10,
    height: height + 10,
    down_x: lastTouch.x,
    down_y: lastTouch.y,
    up_x: e.offsetX,
    up_y: e.offsetY,
  };

  fetch("http://" + window.location.host + "/canvas_touch/", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(canvasTouch),
  });
}

canvas.addEventListener("mousedown", startDrawing);
canvas.addEventListener("mousemove", mouseMoveEvents);
canvas.addEventListener("mouseup", endDrawing);
