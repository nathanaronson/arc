const EXAMPLES = {
    "hello": `var greeting = "hello" + ", " + "web";
print(greeting);
print(type_of(greeting));
`,
    "fibonacci": `fun fib(n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

var start = clock();
print(fib(20));
print("elapsed milliseconds:");
print(clock() - start);
`,
    "loops": `for (var i = 1; i <= 5; i = i + 1) {
    print(i * i);
}

var countdown = 3;
while (countdown > 0) {
    print(countdown);
    countdown = countdown - 1;
}
print("liftoff");
`,
    "errors": `print("before the error");
print(1 + true);
`,
};

const sourceEl = document.getElementById("source");
const gutterEl = document.getElementById("gutter");
const runEl = document.getElementById("run");
const examplesEl = document.getElementById("examples");
const stdoutEl = document.getElementById("stdout");
const stderrEl = document.getElementById("stderr");
const statusEl = document.getElementById("status");
const themeEl = document.getElementById("theme");

for (const name of Object.keys(EXAMPLES)) {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = name;
    examplesEl.append(option);
}

examplesEl.addEventListener("change", () => {
    sourceEl.value = EXAMPLES[examplesEl.value];
    updateGutter();
});

sourceEl.value = EXAMPLES.hello;

function updateGutter() {
    const lines = sourceEl.value.split("\n").length;
    gutterEl.textContent = Array.from({ length: lines }, (_, i) => i + 1).join("\n");
    gutterEl.scrollTop = sourceEl.scrollTop;
}

sourceEl.addEventListener("input", updateGutter);
sourceEl.addEventListener("scroll", () => {
    gutterEl.scrollTop = sourceEl.scrollTop;
});

sourceEl.addEventListener("keydown", (event) => {
    if (event.key === "Tab") {
        event.preventDefault();
        const { selectionStart, selectionEnd } = sourceEl;
        sourceEl.setRangeText("    ", selectionStart, selectionEnd, "end");
        updateGutter();
    }
});

updateGutter();

function applyThemeIcon() {
    themeEl.textContent = document.documentElement.dataset.theme === "dark" ? "☀" : "☾";
}

themeEl.addEventListener("click", () => {
    const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = next;
    localStorage.setItem("theme", next);
    applyThemeIcon();
});

applyThemeIcon();

const isMac = navigator.platform.startsWith("Mac");
runEl.title = isMac ? "⌘⏎" : "Ctrl+Enter";

let worker = null;
let running = false;
let startedAt = 0;

function spawnWorker() {
    runEl.disabled = true;
    worker = new Worker("worker.js", { type: "module" });
    worker.addEventListener("message", (event) => {
        const message = event.data;
        if (message.type === "ready") {
            runEl.disabled = false;
            if (statusEl.textContent === "loading…") {
                statusEl.textContent = "";
            }
            return;
        }
        running = false;
        runEl.textContent = "Run";
        const elapsed = Math.max(1, Math.round(performance.now() - startedAt));
        stdoutEl.textContent = message.stdout;
        stderrEl.textContent = message.stderr;
        statusEl.textContent = message.ok ? `ok · ${elapsed} ms` : "error";
    });
}

spawnWorker();

function runProgram() {
    if (running) {
        worker.terminate();
        running = false;
        runEl.textContent = "Run";
        statusEl.textContent = "stopped";
        spawnWorker();
        return;
    }
    running = true;
    runEl.textContent = "Stop";
    statusEl.textContent = "running…";
    stdoutEl.textContent = "";
    stderrEl.textContent = "";
    startedAt = performance.now();
    worker.postMessage(sourceEl.value);
}

runEl.addEventListener("click", runProgram);

document.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter" && !runEl.disabled) {
        event.preventDefault();
        runProgram();
    }
});
