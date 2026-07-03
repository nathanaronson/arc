import init, { run } from "./pkg/arc.js";

const ready = init().then(() => {
    postMessage({ type: "ready" });
});

onmessage = async (event) => {
    await ready;
    const result = run(event.data);
    postMessage({
        type: "result",
        ok: result.ok,
        stdout: result.stdout,
        stderr: result.stderr,
    });
};
