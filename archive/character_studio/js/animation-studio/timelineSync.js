/** Timeline scrubber synced to AnimationMixer playback. */

export function bindTimelineSync(viewer, slider, timeLabel) {
  let frameId = 0;

  const tick = () => {
    frameId = requestAnimationFrame(tick);
    if (!viewer) return;
    const { time, duration } = viewer.getClipProgress();
    if (duration > 0) {
      slider.max = duration.toFixed(3);
      slider.value = time.toFixed(3);
      timeLabel.textContent = `${time.toFixed(2)}s / ${duration.toFixed(2)}s`;
    } else {
      slider.value = 0;
      timeLabel.textContent = "0.00s / 0.00s";
    }
  };

  tick();
  slider.oninput = () => viewer?.seekClip(parseFloat(slider.value));

  return () => cancelAnimationFrame(frameId);
}
