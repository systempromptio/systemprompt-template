import {
  state, MIN_TIME_MS, RAGE_CLICK_THRESHOLD, RAGE_CLICK_WINDOW_MS,
  getScrollDepth, calcScrollVelocity, detectReadingPattern,
  recordFirst, sendEvent,
} from './engagement-state.js';

export { sendEvent, getScrollDepth };

const buildData = () => {
  const now = Date.now();
  const vt = Math.round(state.visibleTime + (state.isVisible ? now - state.lastVisibilityChange : 0));
  const ht = Math.round(state.hiddenTime + (!state.isVisible ? now - state.lastVisibilityChange : 0));
  return {
    page_url: window.location.pathname,
    time_on_page_ms: Math.round(now - state.pageLoadTime),
    max_scroll_depth: state.maxScrollDepth,
    click_count: state.clickCount,
    focus_time_ms: vt, blur_count: state.tabSwitches || 0,
    tab_switches: state.tabSwitches || 0,
    visible_time_ms: vt, hidden_time_ms: ht,
    time_to_first_interaction_ms: state.firstInteractionTime
      ? Math.round(state.firstInteractionTime - state.pageLoadTime) : null,
    time_to_first_scroll_ms: state.firstScrollTime
      ? Math.round(state.firstScrollTime - state.pageLoadTime) : null,
    scroll_velocity_avg: calcScrollVelocity(),
    scroll_direction_changes: state.scrollDirectionChanges,
    mouse_move_distance_px: Math.round(state.mouseDistance),
    keyboard_events: state.keyboardEvents,
    copy_events: state.copyEvents,
    is_rage_click: state.hasRageClick,
    is_dead_click: state.hasDeadClick,
    reading_pattern: detectReadingPattern(),
  };
};

export const sendPageExit = () => {
  if (!state.dataSent) {
    if (Date.now() - state.pageLoadTime >= MIN_TIME_MS) {
      state.dataSent = true;
      sendEvent('page_exit', buildData());
    }
  }
};

export const handleScroll = () => {
  const depth = getScrollDepth();
  const pos = window.scrollY;
  const now = Date.now();
  if (!state.firstScrollTime) { state.firstScrollTime = now; recordFirst(); }
  if (depth > state.maxScrollDepth) state.maxScrollDepth = depth;
  if (state.scrollPositions.length > 0) {
    const dir = pos > state.scrollPositions[state.scrollPositions.length - 1].position ? 'down' : 'up';
    if (state.lastScrollDirection && dir !== state.lastScrollDirection) state.scrollDirectionChanges++;
    state.lastScrollDirection = dir;
  }
  state.scrollPositions.push({ position: pos, time: now });
  if (state.scrollPositions.length > 50) state.scrollPositions = state.scrollPositions.slice(-50);
};

export const handleClick = (event) => {
  state.clickCount++;
  recordFirst();
  state.clickTimestamps.push(Date.now());
  state.clickTimestamps = state.clickTimestamps.filter((t) => Date.now() - t < RAGE_CLICK_WINDOW_MS);
  if (state.clickTimestamps.length >= RAGE_CLICK_THRESHOLD) state.hasRageClick = true;
  const tgt = event.target;
  const interactive = tgt.tagName === 'A' || tgt.tagName === 'BUTTON' || tgt.tagName === 'INPUT'
    || tgt.closest('a') || tgt.closest('button');
  if (!interactive && state.clickCount > 1) state.hasDeadClick = true;
};

export const handleMouseMove = (event) => {
  if (state.lastMousePosition) {
    const dx = event.clientX - state.lastMousePosition.x;
    const dy = event.clientY - state.lastMousePosition.y;
    state.mouseDistance += Math.sqrt(dx * dx + dy * dy);
  }
  state.lastMousePosition = { x: event.clientX, y: event.clientY };
};

export const handleKeydown = () => { state.keyboardEvents++; recordFirst(); };
export const handleCopy = () => { state.copyEvents++; recordFirst(); };

export const handleVisibilityChange = () => {
  const now = Date.now();
  const elapsed = now - state.lastVisibilityChange;
  if (state.isVisible) { state.visibleTime += elapsed; state.focusTime += elapsed; }
  else { state.hiddenTime += elapsed; }
  state.isVisible = !document.hidden;
  state.lastVisibilityChange = now;
  if (document.hidden) { state.tabSwitches++; sendPageExit(); }
};

export const handleFocus = () => {
  if (!state.isVisible) {
    state.hiddenTime += Date.now() - state.lastVisibilityChange;
    state.lastVisibilityChange = Date.now();
    state.isVisible = true;
  }
};

export const handleBlur = () => {
  state.blurCount++;
  if (state.isVisible) {
    const now = Date.now();
    state.visibleTime += now - state.lastVisibilityChange;
    state.focusTime += now - state.lastVisibilityChange;
    state.lastVisibilityChange = now;
    state.isVisible = false;
  }
};
