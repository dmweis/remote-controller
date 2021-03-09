const canvas = document.getElementById("navigation_canvas");
const ctx = canvas.getContext("2d");

let map = { width: 1, height: 1 };

let width = 0.0;
let height = 0.0;

let viewport_top = 0.0;
let viewport_left = 0.0;
let viewport_height = 0.0;
let viewport_width = 0.0;

function clear() {
  const scale = Math.min(height / map.height, width / map.width);
  viewport_width = map.width * scale;
  viewport_height = map.height * scale;
  ctx.fillStyle = "#314755";
  ctx.fillRect(0, 0, width, height);
  ctx.fillStyle = "pink";
  viewport_top = height / 2 - viewport_height / 2;
  viewport_left = width / 2 - viewport_width / 2;
  ctx.fillRect(viewport_left, viewport_top, viewport_width, viewport_height);
}

function updateCanvasSize() {
  canvas.width = canvas.clientWidth;
  canvas.height = canvas.clientHeight;

  width = canvas.width;
  height = canvas.height;
  clear();
}

updateCanvasSize();

window.addEventListener("resize", updateCanvasSize);

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

  const canvasTouch = {
    width: viewport_width,
    height: viewport_height,
    down_x: lastTouch.x - viewport_left,
    down_y: lastTouch.y - viewport_top,
    up_x: e.offsetX - viewport_left,
    up_y: e.offsetY - viewport_top,
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

fetch("http://" + window.location.host + "/map").then((response) => {
  response.json().then((data) => {
    map = data;
    clear();
  });
});
