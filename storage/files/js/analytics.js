(function() {
  'use strict';

  const ENDPOINT = '/track/engagement';
  const BATCH_ENDPOINT = '/track/engagement/batch';
  const MIN_TIME_MS = 5000;
  const RAGE_CLICK_THRESHOLD = 3;
  const RAGE_CLICK_WINDOW_MS = 500;
  const SCROLL_MILESTONES = [25, 50, 75, 90, 100];

  const state = {
    pageLoadTime: Date.now(),
    firstInteractionTime: null,
    firstScrollTime: null,
    maxScrollDepth: 0,
    scrollPositions: [],
    scrollDirectionChanges: 0,
    lastScrollDirection: null,
    clickCount: 0,
    clickTimestamps: [],
    hasRageClick: false,
    hasDeadClick: false,
    mouseDistance: 0,
    lastMousePosition: null,
    keyboardEvents: 0,
    copyEvents: 0,
    focusTime: 0,
    blurCount: 0,
    tabSwitches: 0,
    visibleTime: 0,
    hiddenTime: 0,
    lastVisibilityChange: Date.now(),
    isVisible: !document.hidden,
    dataSent: false,
    pageViewSent: false,
    scrollMilestonesSent: {},
    linkClicks: []
  };

  function getScrollDepth() {
    const windowHeight = window.innerHeight;
    const documentHeight = Math.max(
      document.body.scrollHeight,
      document.body.offsetHeight,
      document.documentElement.scrollHeight,
      document.documentElement.offsetHeight
    );
    const scrollTop = window.scrollY || document.documentElement.scrollTop;

    if (documentHeight <= windowHeight) {
      return 100;
    }

    return Math.min(100, Math.round((scrollTop + windowHeight) / documentHeight * 100));
  }

  function calculateScrollVelocity() {
    if (state.scrollPositions.length < 2) {
      return null;
    }

    const recent = state.scrollPositions.slice(-10);
    let totalVelocity = 0;

    for (let i = 1; i < recent.length; i++) {
      const timeDiff = recent[i].time - recent[i - 1].time;
      const posDiff = Math.abs(recent[i].position - recent[i - 1].position);
      if (timeDiff > 0) {
        totalVelocity += posDiff / timeDiff;
      }
    }

    return Math.round(totalVelocity / (recent.length - 1) * 1000);
  }

  function detectRageClick(timestamp) {
    state.clickTimestamps.push(timestamp);

    const recentClicks = state.clickTimestamps.filter(function(t) {
      return timestamp - t < RAGE_CLICK_WINDOW_MS;
    });

    state.clickTimestamps = recentClicks;

    if (recentClicks.length >= RAGE_CLICK_THRESHOLD) {
      state.hasRageClick = true;
    }
  }

  function detectReadingPattern() {
    const timeOnPage = Date.now() - state.pageLoadTime;
    const scrollDepth = state.maxScrollDepth;

    if (timeOnPage < 10000 && scrollDepth < 25) {
      return 'bounce';
    }

    if (scrollDepth > 75 && timeOnPage > 30000) {
      return 'engaged';
    }

    if (scrollDepth > 50 && timeOnPage > 15000) {
      return 'reader';
    }

    if (scrollDepth > 30 && timeOnPage < 20000) {
      return 'scanner';
    }

    return 'skimmer';
  }

  function buildEngagementData() {
    const now = Date.now();
    const timeOnPage = now - state.pageLoadTime;
    const visibleTime = Math.round(state.visibleTime + (state.isVisible ? now - state.lastVisibilityChange : 0));
    const hiddenTime = Math.round(state.hiddenTime + (!state.isVisible ? now - state.lastVisibilityChange : 0));

    return {
      page_url: window.location.pathname,
      time_on_page_ms: Math.round(timeOnPage),
      max_scroll_depth: state.maxScrollDepth,
      click_count: state.clickCount,
      focus_time_ms: visibleTime,
      blur_count: state.tabSwitches || 0,
      tab_switches: state.tabSwitches || 0,
      visible_time_ms: visibleTime,
      hidden_time_ms: hiddenTime,
      time_to_first_interaction_ms: state.firstInteractionTime
        ? Math.round(state.firstInteractionTime - state.pageLoadTime)
        : null,
      time_to_first_scroll_ms: state.firstScrollTime
        ? Math.round(state.firstScrollTime - state.pageLoadTime)
        : null,
      scroll_velocity_avg: calculateScrollVelocity(),
      scroll_direction_changes: state.scrollDirectionChanges,
      mouse_move_distance_px: Math.round(state.mouseDistance),
      keyboard_events: state.keyboardEvents,
      copy_events: state.copyEvents,
      is_rage_click: state.hasRageClick,
      is_dead_click: state.hasDeadClick,
      reading_pattern: detectReadingPattern()
    };
  }

  function sendEvent(eventType, eventData) {
    const payload = {
      page_url: window.location.pathname,
      data: {
        ...eventData,
        event_type: eventType
      }
    };

    const jsonPayload = JSON.stringify(payload);

    if (navigator.sendBeacon) {
      const blob = new Blob([jsonPayload], { type: 'application/json' });
      navigator.sendBeacon(ENDPOINT, blob);
    } else {
      const xhr = new XMLHttpRequest();
      xhr.open('POST', ENDPOINT, true);
      xhr.setRequestHeader('Content-Type', 'application/json');
      xhr.withCredentials = true;
      xhr.send(jsonPayload);
    }
  }

  function sendPageView() {
    if (state.pageViewSent) return;
    state.pageViewSent = true;

    sendEvent('page_view', {
      referrer: document.referrer || null,
      title: document.title
    });
  }

  function sendPageExit() {
    if (state.dataSent) return;

    const timeOnPage = Date.now() - state.pageLoadTime;
    if (timeOnPage < MIN_TIME_MS) return;

    state.dataSent = true;
    sendEvent('page_exit', buildEngagementData());
  }

  function sendScrollMilestone(milestone) {
    if (state.scrollMilestonesSent[milestone]) return;
    state.scrollMilestonesSent[milestone] = true;

    sendEvent('scroll', {
      depth: state.maxScrollDepth,
      milestone: milestone,
      direction: state.lastScrollDirection || 'down',
      velocity: calculateScrollVelocity()
    });
  }

  function sendLinkClick(targetUrl, linkText, isExternal) {
    sendEvent('link_click', {
      target_url: targetUrl,
      link_text: linkText ? linkText.substring(0, 100) : null,
      is_external: isExternal
    });
  }

  function recordFirstInteraction() {
    if (!state.firstInteractionTime) {
      state.firstInteractionTime = Date.now();
    }
  }

  function handleScroll() {
    const currentDepth = getScrollDepth();
    const currentPosition = window.scrollY;
    const currentTime = Date.now();

    if (!state.firstScrollTime) {
      state.firstScrollTime = currentTime;
      recordFirstInteraction();
    }

    if (currentDepth > state.maxScrollDepth) {
      state.maxScrollDepth = currentDepth;

      for (let i = 0; i < SCROLL_MILESTONES.length; i++) {
        const milestone = SCROLL_MILESTONES[i];
        if (currentDepth >= milestone && !state.scrollMilestonesSent[milestone]) {
          sendScrollMilestone(milestone);
        }
      }
    }

    if (state.scrollPositions.length > 0) {
      const lastPosition = state.scrollPositions[state.scrollPositions.length - 1].position;
      const direction = currentPosition > lastPosition ? 'down' : 'up';

      if (state.lastScrollDirection && direction !== state.lastScrollDirection) {
        state.scrollDirectionChanges++;
      }

      state.lastScrollDirection = direction;
    }

    state.scrollPositions.push({ position: currentPosition, time: currentTime });

    if (state.scrollPositions.length > 50) {
      state.scrollPositions = state.scrollPositions.slice(-50);
    }
  }

  function handleClick(event) {
    state.clickCount++;
    recordFirstInteraction();
    detectRageClick(Date.now());

    const target = event.target;
    const link = target.tagName === 'A' ? target : target.closest('a');

    if (link && link.href) {
      const isExternal = link.hostname !== window.location.hostname;
      const linkText = link.textContent || link.innerText;
      sendLinkClick(link.href, linkText, isExternal);
    }

    const isInteractive = target.tagName === 'A' ||
                         target.tagName === 'BUTTON' ||
                         target.tagName === 'INPUT' ||
                         target.closest('a') ||
                         target.closest('button');

    if (!isInteractive && state.clickCount > 1) {
      state.hasDeadClick = true;
    }
  }

  function handleMouseMove(event) {
    if (state.lastMousePosition) {
      const dx = event.clientX - state.lastMousePosition.x;
      const dy = event.clientY - state.lastMousePosition.y;
      state.mouseDistance += Math.sqrt(dx * dx + dy * dy);
    }

    state.lastMousePosition = { x: event.clientX, y: event.clientY };
  }

  function handleKeydown() {
    state.keyboardEvents++;
    recordFirstInteraction();
  }

  function handleCopy() {
    state.copyEvents++;
    recordFirstInteraction();
  }

  function handleVisibilityChange() {
    const now = Date.now();
    const elapsed = now - state.lastVisibilityChange;

    if (state.isVisible) {
      state.visibleTime += elapsed;
      state.focusTime += elapsed;
    } else {
      state.hiddenTime += elapsed;
    }

    state.isVisible = !document.hidden;
    state.lastVisibilityChange = now;

    if (document.hidden) {
      state.tabSwitches++;
      sendPageExit();
    }
  }

  function handleFocus() {
    if (!state.isVisible) {
      const now = Date.now();
      state.hiddenTime += now - state.lastVisibilityChange;
      state.lastVisibilityChange = now;
      state.isVisible = true;
    }
  }

  function handleBlur() {
    state.blurCount++;

    if (state.isVisible) {
      const now = Date.now();
      state.visibleTime += now - state.lastVisibilityChange;
      state.focusTime += now - state.lastVisibilityChange;
      state.lastVisibilityChange = now;
      state.isVisible = false;
    }
  }

  function handlePageHide() {
    sendPageExit();
  }

  let scrollTimeout;
  function throttledScroll() {
    if (!scrollTimeout) {
      scrollTimeout = setTimeout(function() {
        scrollTimeout = null;
        handleScroll();
      }, 100);
    }
  }

  let mouseMoveTimeout;
  function throttledMouseMove(event) {
    if (!mouseMoveTimeout) {
      mouseMoveTimeout = setTimeout(function() {
        mouseMoveTimeout = null;
      }, 50);
      handleMouseMove(event);
    }
  }

  sendPageView();

  window.addEventListener('scroll', throttledScroll, { passive: true });
  window.addEventListener('click', handleClick, { passive: true });
  window.addEventListener('mousemove', throttledMouseMove, { passive: true });
  window.addEventListener('keydown', handleKeydown, { passive: true });
  document.addEventListener('copy', handleCopy);
  document.addEventListener('visibilitychange', handleVisibilityChange);
  window.addEventListener('focus', handleFocus);
  window.addEventListener('blur', handleBlur);
  window.addEventListener('pagehide', handlePageHide);
  window.addEventListener('beforeunload', sendPageExit);

  handleScroll();
})();
