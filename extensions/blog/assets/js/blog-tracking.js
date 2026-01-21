(function() {
  'use strict';

  var ENDPOINT = '/api/v1/analytics/events';
  var BATCH_ENDPOINT = '/api/v1/analytics/events/batch';
  var MIN_TIME_MS = 5000;
  var RAGE_CLICK_THRESHOLD = 3;
  var RAGE_CLICK_WINDOW_MS = 500;
  var SCROLL_MILESTONES = [25, 50, 75, 90, 100];

  var state = {
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
    var windowHeight = window.innerHeight;
    var documentHeight = Math.max(
      document.body.scrollHeight,
      document.body.offsetHeight,
      document.documentElement.scrollHeight,
      document.documentElement.offsetHeight
    );
    var scrollTop = window.scrollY || document.documentElement.scrollTop;

    if (documentHeight <= windowHeight) {
      return 100;
    }

    return Math.min(100, Math.round((scrollTop + windowHeight) / documentHeight * 100));
  }

  function calculateScrollVelocity() {
    if (state.scrollPositions.length < 2) {
      return null;
    }

    var recent = state.scrollPositions.slice(-10);
    var totalVelocity = 0;

    for (var i = 1; i < recent.length; i++) {
      var timeDiff = recent[i].time - recent[i - 1].time;
      var posDiff = Math.abs(recent[i].position - recent[i - 1].position);
      if (timeDiff > 0) {
        totalVelocity += posDiff / timeDiff;
      }
    }

    return Math.round(totalVelocity / (recent.length - 1) * 1000);
  }

  function detectRageClick(timestamp) {
    state.clickTimestamps.push(timestamp);

    var recentClicks = state.clickTimestamps.filter(function(t) {
      return timestamp - t < RAGE_CLICK_WINDOW_MS;
    });

    state.clickTimestamps = recentClicks;

    if (recentClicks.length >= RAGE_CLICK_THRESHOLD) {
      state.hasRageClick = true;
    }
  }

  function detectReadingPattern() {
    var timeOnPage = Date.now() - state.pageLoadTime;
    var scrollDepth = state.maxScrollDepth;

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
    var now = Date.now();
    var timeOnPage = now - state.pageLoadTime;

    return {
      scroll_depth: state.maxScrollDepth,
      time_on_page_ms: timeOnPage,
      time_to_first_interaction_ms: state.firstInteractionTime
        ? state.firstInteractionTime - state.pageLoadTime
        : null,
      time_to_first_scroll_ms: state.firstScrollTime
        ? state.firstScrollTime - state.pageLoadTime
        : null,
      scroll_velocity_avg: calculateScrollVelocity(),
      scroll_direction_changes: state.scrollDirectionChanges,
      mouse_move_distance_px: Math.round(state.mouseDistance),
      keyboard_events: state.keyboardEvents,
      copy_events: state.copyEvents,
      visible_time_ms: state.visibleTime + (state.isVisible ? now - state.lastVisibilityChange : 0),
      hidden_time_ms: state.hiddenTime + (!state.isVisible ? now - state.lastVisibilityChange : 0),
      click_count: state.clickCount,
      is_rage_click: state.hasRageClick,
      is_dead_click: state.hasDeadClick,
      reading_pattern: detectReadingPattern()
    };
  }

  function sendEvent(eventType, data) {
    var payload = {
      event_type: eventType,
      page_url: window.location.pathname,
      referrer: document.referrer || null,
      data: data || {}
    };

    var jsonPayload = JSON.stringify(payload);

    if (navigator.sendBeacon) {
      var blob = new Blob([jsonPayload], { type: 'application/json' });
      navigator.sendBeacon(ENDPOINT, blob);
    } else {
      var xhr = new XMLHttpRequest();
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

    var timeOnPage = Date.now() - state.pageLoadTime;
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
    var currentDepth = getScrollDepth();
    var currentPosition = window.scrollY;
    var currentTime = Date.now();

    if (!state.firstScrollTime) {
      state.firstScrollTime = currentTime;
      recordFirstInteraction();
    }

    if (currentDepth > state.maxScrollDepth) {
      state.maxScrollDepth = currentDepth;

      for (var i = 0; i < SCROLL_MILESTONES.length; i++) {
        var milestone = SCROLL_MILESTONES[i];
        if (currentDepth >= milestone && !state.scrollMilestonesSent[milestone]) {
          sendScrollMilestone(milestone);
        }
      }
    }

    if (state.scrollPositions.length > 0) {
      var lastPosition = state.scrollPositions[state.scrollPositions.length - 1].position;
      var direction = currentPosition > lastPosition ? 'down' : 'up';

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

    var target = event.target;
    var link = target.tagName === 'A' ? target : target.closest('a');

    if (link && link.href) {
      var isExternal = link.hostname !== window.location.hostname;
      var linkText = link.textContent || link.innerText;
      sendLinkClick(link.href, linkText, isExternal);
    }

    var isInteractive = target.tagName === 'A' ||
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
      var dx = event.clientX - state.lastMousePosition.x;
      var dy = event.clientY - state.lastMousePosition.y;
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
    var now = Date.now();
    var elapsed = now - state.lastVisibilityChange;

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
      var now = Date.now();
      state.hiddenTime += now - state.lastVisibilityChange;
      state.lastVisibilityChange = now;
      state.isVisible = true;
    }
  }

  function handleBlur() {
    state.blurCount++;

    if (state.isVisible) {
      var now = Date.now();
      state.visibleTime += now - state.lastVisibilityChange;
      state.focusTime += now - state.lastVisibilityChange;
      state.lastVisibilityChange = now;
      state.isVisible = false;
    }
  }

  function handlePageHide() {
    sendPageExit();
  }

  var scrollTimeout;
  function throttledScroll() {
    if (!scrollTimeout) {
      scrollTimeout = setTimeout(function() {
        scrollTimeout = null;
        handleScroll();
      }, 100);
    }
  }

  var mouseMoveTimeout;
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
