'use strict';

const descriptions = {
  data: 'Ask an agent with access to your files to read them and return what you need.',
  tools: 'Send a task to a device with the shell tools, scripts or test environment it needs.',
  compute: 'Use a local model or send a new task to a device with enough available CPU and RAM.'
};
const stage = document.querySelector('.network-stage');
function drawConnections() {
  const bounds = stage.getBoundingClientRect();
  const center = document.querySelector('.agent-center').getBoundingClientRect();
  const start = { x: center.left - bounds.left + center.width / 2, y: center.top - bounds.top + center.height / 2 };
  document.querySelector('.connections').setAttribute('viewBox', `0 0 ${bounds.width} ${bounds.height}`);
  Object.keys(descriptions).forEach(key => {
    const box = document.querySelector(`[data-node="${key}"]`).getBoundingClientRect();
    const end = { x: box.left - bounds.left + box.width / 2, y: box.top - bounds.top + box.height / 2 };
    const middleY = (start.y + end.y) / 2;
    document.querySelector(`.${key}-wire`).setAttribute('d', `M${start.x} ${start.y} V${middleY} H${end.x} V${end.y}`);
    const marker = document.querySelector(`.${key}-waypoint`);
    marker.setAttribute('cx', (start.x + end.x) / 2);
    marker.setAttribute('cy', middleY);
  });
}
new ResizeObserver(drawConnections).observe(stage);
document.querySelectorAll('[data-capability]').forEach(button => {
  button.addEventListener('click', () => {
    const capability = button.dataset.capability;
    document.querySelector('.network-stage').dataset.active = capability;
    document.querySelectorAll('[data-capability]').forEach(item => item.setAttribute('aria-pressed', String(item === button)));
    document.querySelectorAll('[data-node]').forEach(node => node.classList.toggle('selected', node.dataset.node === capability));
    document.querySelector('.network-explanation').textContent = descriptions[capability];
  });
});

const platforms = {
  arm: { archive: 'aarch64-apple-darwin', label: 'Apple Silicon', note: 'Signed and notarized macOS build. Python 3.11+ is required for installation.' },
  intel: { archive: 'x86_64-apple-darwin', label: 'Intel Mac', note: 'Signed and notarized macOS build. Python 3.11+ is required for installation.' },
  linux: { archive: 'x86_64-unknown-linux-gnu', label: 'Linux x86-64', note: 'Requires glibc 2.39+ (Ubuntu 24.04 baseline) and Python 3.11+. Other systems can build from source.' }
};
document.querySelector('#platform').addEventListener('change', event => {
  const platform = platforms[event.target.value];
  const url = `https://github.com/priyankark/citta/releases/download/v0.1.0/citta-0.1.0-${platform.archive}.tar.gz`;
  const download = document.querySelector('#download-link');
  download.href = url;
  download.replaceChildren(document.createTextNode(`Download for ${platform.label} `));
  const arrow = document.createElement('span');
  arrow.setAttribute('aria-hidden', 'true');
  arrow.textContent = '↓';
  download.append(arrow);
  document.querySelector('#checksum-link').href = `${url}.sha256`;
  document.querySelector('#platform-note').textContent = platform.note;
});

document.querySelector('#copy-commands').addEventListener('click', async () => {
  const status = document.querySelector('#copy-status');
  try {
    await navigator.clipboard.writeText('citta setup\ncitta doctor');
    status.textContent = 'Copied. Run these on each device after installing Citta.';
  } catch {
    status.textContent = 'Copy is unavailable. Select the two commands above to copy them.';
  }
});

document.querySelector('#play-video').addEventListener('click', () => {
  const frame = document.createElement('iframe');
  frame.src = 'https://www.youtube-nocookie.com/embed/EPdPu5djxSQ?autoplay=1&rel=0';
  frame.title = 'Citta: agents sharing tools, data and compute';
  frame.allow = 'autoplay; encrypted-media; picture-in-picture; fullscreen';
  frame.allowFullscreen = true;
  frame.referrerPolicy = 'strict-origin-when-cross-origin';
  document.querySelector('#video-shell').replaceChildren(frame);
  frame.focus();
});
