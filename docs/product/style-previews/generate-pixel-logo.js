// Generate pixel logo SVG from the exact bezier curves
// Run: node generate-pixel-logo.js > logo-pixel.svg

function cubicBezier(p0, p1, p2, p3, t) {
  const m = 1 - t;
  return m*m*m*p0 + 3*m*m*t*p1 + 3*m*t*t*p2 + t*t*t*p3;
}

function traceCurve(segs) {
  const pts = [];
  const per = 150;
  for (const s of segs) {
    for (let i = 0; i < per; i++) {
      const t = i / per;
      pts.push([
        cubicBezier(s[0], s[2], s[4], s[6], t),
        cubicBezier(s[1], s[3], s[5], s[7], t)
      ]);
    }
  }
  return pts;
}

const topCurve = traceCurve([
  [2,14, 16,8, 32,22, 44,38],
  [44,38, 52,50, 62,52, 72,48],
  [72,48, 76,46, 78,44, 78,44],
]);

const midCurve = traceCurve([
  [2,40, 14,28, 30,26, 42,34],
  [42,34, 54,42, 60,46, 70,43],
  [70,43, 75,43, 78,44, 78,44],
]);

const botCurve = traceCurve([
  [2,66, 14,58, 28,46, 42,40],
  [42,40, 52,36, 62,40, 70,44],
  [70,44, 75,46, 78,44, 78,44],
]);

const SIZE = 48;
const margin = 2;
const dW = SIZE - 4;
const dH = SIZE - 4;

function toPixel(px, py) {
  return [
    Math.round(margin + (px / 80) * dW),
    Math.round(margin + (py / 80) * dH)
  ];
}

// Collect pixels per curve
const pixels = {};

function addPixel(x, y, r, g, b, a) {
  if (x < 0 || x >= SIZE || y < 0 || y >= SIZE) return;
  const key = `${x},${y}`;
  if (!pixels[key]) {
    pixels[key] = { r: 0, g: 0, b: 0, a: 0 };
  }
  const p = pixels[key];
  const srcA = a / 255;
  const dstA = p.a / 255;
  const outA = srcA + dstA * (1 - srcA);
  if (outA > 0) {
    p.r = Math.round((r * srcA + p.r * dstA * (1 - srcA)) / outA);
    p.g = Math.round((g * srcA + p.g * dstA * (1 - srcA)) / outA);
    p.b = Math.round((b * srcA + p.b * dstA * (1 - srcA)) / outA);
    p.a = Math.round(outA * 255);
  }
}

const layers = [
  { pts: botCurve, color: [8, 145, 178], alpha: 230 },
  { pts: midCurve, color: [6, 182, 212], alpha: 220 },
  { pts: topCurve, color: [103, 232, 249], alpha: 210 },
];

layers.forEach(layer => {
  const step = Math.max(1, Math.floor(layer.pts.length / (SIZE * 4)));
  for (let i = 0; i < layer.pts.length; i += step) {
    const [px, py] = toPixel(layer.pts[i][0], layer.pts[i][1]);
    addPixel(px, py, layer.color[0], layer.color[1], layer.color[2], layer.alpha);
  }
});

// Bright tip
const [tx, ty] = toPixel(78, 44);
addPixel(tx, ty, 165, 243, 252, 255);

// Generate SVG
let rects = '';
for (const [key, p] of Object.entries(pixels)) {
  const [x, y] = key.split(',').map(Number);
  const opacity = (p.a / 255).toFixed(2);
  rects += `  <rect x="${x}" y="${y}" width="1" height="1" fill="rgb(${p.r},${p.g},${p.b})" opacity="${opacity}"/>\n`;
}

const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48" shape-rendering="crispEdges">
<rect width="48" height="48" fill="#06080c"/>
${rects}</svg>`;

process.stdout.write(svg);
