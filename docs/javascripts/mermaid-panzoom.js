/*
 * Mermaid pan/zoom viewer adapted from poorman-aws.
 * Copyright (c) 2026 Ken C. Demanawa; licensed under the MIT License.
 */
(function () {
  "use strict";

  var MIN_SCALE = 0.5;
  var MAX_SCALE = 4;
  var SCALE_STEP = 0.2;
  var ENHANCED = "data-mermaid-panzoom-ready";
  var WAITING = "data-mermaid-panzoom-waiting";

  // Zensical renders Mermaid diagrams into a closed shadow root. Open only
  // Mermaid hosts so the SVG can be used as the pan/zoom transform target.
  var attachShadow = Element.prototype.attachShadow;
  Element.prototype.attachShadow = function (init) {
    if (init && this.classList && this.classList.contains("mermaid") && init.mode === "closed") {
      init = Object.assign({}, init, { mode: "open" });
    }

    return attachShadow.call(this, init);
  };

  function clamp(value, minimum, maximum) {
    return Math.min(Math.max(value, minimum), maximum);
  }

  function updateScale(state, nextScale, originX, originY) {
    var scale = clamp(nextScale, MIN_SCALE, MAX_SCALE);
    var ratio = scale / state.scale;

    state.x = originX - (originX - state.x) * ratio;
    state.y = originY - (originY - state.y) * ratio;
    state.scale = scale;
  }

  function getDiagramOrigin(state, viewport) {
    var viewportBounds = viewport.getBoundingClientRect();
    var diagramBounds = state.diagram.getBoundingClientRect();

    return {
      x: diagramBounds.left - viewportBounds.left - state.x,
      y: diagramBounds.top - viewportBounds.top - state.y
    };
  }

  function zoomAt(state, viewport, nextScale, clientX, clientY) {
    if (!state.diagram) {
      return;
    }

    var viewportBounds = viewport.getBoundingClientRect();
    var diagramOrigin = getDiagramOrigin(state, viewport);
    var originX = clientX - viewportBounds.left - diagramOrigin.x;
    var originY = clientY - viewportBounds.top - diagramOrigin.y;
    updateScale(state, nextScale, originX, originY);
  }

  function render(state) {
    if (state.diagram) {
      state.diagram.style.setProperty(
        "transform",
        "translate(" + state.x + "px, " + state.y + "px) scale(" + state.scale + ")",
        "important"
      );
    }
    var zoomPercentage = Math.round(state.scale * 100);
    state.zoom.textContent = zoomPercentage + "%";
    state.zoomOut.disabled = state.scale <= MIN_SCALE;
    state.zoomIn.disabled = state.scale >= MAX_SCALE;
    state.reset.disabled = zoomPercentage === 100;
  }

  function findSvg(mermaid) {
    var roots = [mermaid];

    while (roots.length > 0) {
      var root = roots.shift();
      if (root.shadowRoot) {
        roots.push(root.shadowRoot);
      }

      var svg = root.querySelector && root.querySelector("svg");
      if (svg) {
        return svg;
      }

      if (!root.querySelectorAll) {
        continue;
      }

      root.querySelectorAll("*").forEach(function (element) {
        if (element.shadowRoot) {
          roots.push(element.shadowRoot);
        }
      });
    }

    return null;
  }

  function waitForSvg(mermaid, onSvg) {
    if (mermaid.hasAttribute(WAITING)) {
      return;
    }

    mermaid.setAttribute(WAITING, "");
    var attempts = 0;
    var finish = function () {
      var svg = findSvg(mermaid);
      if (!svg) {
        return;
      }

      window.clearInterval(retry);
      if (observer) {
        observer.disconnect();
      }
      mermaid.removeAttribute(WAITING);
      onSvg(svg);
    };
    var observer = mermaid.shadowRoot ? new MutationObserver(finish) : null;
    var retry = window.setInterval(function () {
      finish();
      attempts += 1;
      if (attempts >= 40) {
        window.clearInterval(retry);
        mermaid.removeAttribute(WAITING);
        if (observer) {
          observer.disconnect();
        }
      }
    }, 250);
    if (observer) {
      observer.observe(mermaid.shadowRoot, { childList: true, subtree: true });
    }
  }

  function enhance(mermaid) {
    if (mermaid.tagName === "PRE" || mermaid.hasAttribute(ENHANCED)) {
      return;
    }

    var originalParent = mermaid.parentNode;
    if (!originalParent) {
      return;
    }

    mermaid.setAttribute(ENHANCED, "");

    var viewport = document.createElement("div");
    viewport.className = "mermaid-panzoom__viewport";
    viewport.title = "Click and hold the left mouse button to pan the diagram";
    viewport.setAttribute("aria-label", "Diagram canvas. Click and hold the left mouse button to pan.");

    var wrapper = document.createElement("div");
    wrapper.className = "mermaid-panzoom";
    wrapper.tabIndex = 0;
    wrapper.setAttribute("role", "group");
    wrapper.setAttribute("aria-label", "Interactive Mermaid diagram");

    var toolbar = document.createElement("div");
    toolbar.className = "mermaid-panzoom__toolbar";

    function button(label, icon, className, handler) {
      var control = document.createElement("button");
      control.type = "button";
      control.className = "mermaid-panzoom__button " + className;
      control.setAttribute("aria-label", label);
      control.title = label;
      var iconElement = document.createElement("span");
      iconElement.className = "mermaid-panzoom__icon";
      iconElement.setAttribute("aria-hidden", "true");
      iconElement.textContent = icon;
      control.appendChild(iconElement);
      control.iconElement = iconElement;
      control.addEventListener("click", handler);
      return control;
    }

    var state = {
      diagram: null,
      scale: 1,
      x: 0,
      y: 0,
      dragging: false,
      lastX: 0,
      lastY: 0,
      pointers: new Map(),
      pinchDistance: 0,
      pinchScale: 1
    };

    var panHint = "Click and hold the left mouse button to pan the diagram";
    var zoomHint = "Use the mouse wheel or your touchpad to zoom in or out";

    function updateCanvasHint(event) {
      if (state.dragging || !state.diagram) {
        return;
      }

      var bounds = state.diagram.getBoundingClientRect();
      var overDiagram = event.clientX >= bounds.left && event.clientX <= bounds.right &&
        event.clientY >= bounds.top && event.clientY <= bounds.bottom;
      viewport.title = overDiagram ? panHint : zoomHint;
    }

    state.zoomOut = button("Zoom out", "−", "mermaid-panzoom__zoom-out", function () {
      zoomAt(
        state,
        viewport,
        state.scale - SCALE_STEP,
        viewport.getBoundingClientRect().left + viewport.clientWidth / 2,
        viewport.getBoundingClientRect().top + viewport.clientHeight / 2
      );
      render(state);
    });

    state.zoom = document.createElement("span");
    state.zoom.className = "mermaid-panzoom__zoom-level";
    state.zoom.setAttribute("aria-live", "polite");

    state.zoomIn = button("Zoom in", "+", "mermaid-panzoom__zoom-in", function () {
      zoomAt(
        state,
        viewport,
        state.scale + SCALE_STEP,
        viewport.getBoundingClientRect().left + viewport.clientWidth / 2,
        viewport.getBoundingClientRect().top + viewport.clientHeight / 2
      );
      render(state);
    });

    function resetAndCenter() {
      if (!state.diagram) {
        return;
      }

      var transition = state.diagram.style.transition;
      state.diagram.style.transition = "none";
      state.scale = 1;
      state.x = 0;
      state.y = 0;
      render(state);

      // Wait for the container's new dimensions after a fullscreen transition
      // before calculating the centered translation.
      requestAnimationFrame(function () {
        var diagramBounds = state.diagram.getBoundingClientRect();
        var viewportBounds = viewport.getBoundingClientRect();
        state.x = (viewportBounds.width - diagramBounds.width) / 2;
        state.y = (viewportBounds.height - diagramBounds.height) / 2;
        render(state);
        state.diagram.style.transition = transition;
      });
    }

    var reset = button("Reset diagram view", "↺", "mermaid-panzoom__reset", function () {
      resetAndCenter();
    });
    state.reset = reset;

    function attachSvg(svg) {
      if (state.diagram === svg) {
        return;
      }

      state.diagram = svg;
      state.diagram.style.transformBox = "fill-box";
      state.diagram.style.transformOrigin = "0 0";
      state.diagram.style.transition = "transform 120ms ease-out";
      state.diagram.setAttribute("draggable", "false");
      resetAndCenter();
    }

    var fullscreen = button("Open diagram fullscreen", "⛶", "mermaid-panzoom__fullscreen", function () {
      if (document.fullscreenElement === wrapper) {
        if (document.exitFullscreen) {
          document.exitFullscreen();
        }
      } else if (wrapper.requestFullscreen) {
        wrapper.requestFullscreen();
      }
    });

    function updateFullscreenControl() {
      var isFullscreen = document.fullscreenElement === wrapper;
      var label = isFullscreen ? "Exit diagram fullscreen" : "Open diagram fullscreen";
      fullscreen.setAttribute("aria-label", label);
      fullscreen.title = label;
      fullscreen.iconElement.textContent = isFullscreen ? "⤡" : "⛶";
      resetAndCenter();
    }

    toolbar.append(state.zoomOut, state.zoom, state.zoomIn, reset, fullscreen);
    wrapper.append(toolbar, viewport);
    originalParent.replaceChild(wrapper, mermaid);
    viewport.appendChild(mermaid);
    document.addEventListener("fullscreenchange", function () {
      updateFullscreenControl();
    });
    updateFullscreenControl();

    var svg = findSvg(mermaid);
    if (svg) {
      attachSvg(svg);
    } else {
      waitForSvg(mermaid, attachSvg);
    }

    viewport.addEventListener("wheel", function (event) {
      if (!state.diagram) {
        return;
      }

      event.preventDefault();
      var factor = event.deltaY < 0 ? 1 + SCALE_STEP : 1 - SCALE_STEP;
      zoomAt(state, viewport, state.scale * factor, event.clientX, event.clientY);
      render(state);
    }, { passive: false });

    viewport.addEventListener("pointerdown", function (event) {
      var path = event.composedPath();
      var overControl = path.some(function (target) {
        return target instanceof Element && target.closest("button");
      });

      if (!state.diagram || event.button !== 0 || overControl) {
        return;
      }

      event.preventDefault();
      state.pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
      viewport.setPointerCapture(event.pointerId);

      if (state.pointers.size === 2) {
        var pinchPoints = Array.from(state.pointers.values());
        state.pinchDistance = Math.hypot(
          pinchPoints[0].x - pinchPoints[1].x,
          pinchPoints[0].y - pinchPoints[1].y
        );
        state.pinchScale = state.scale;
        state.dragging = false;
        viewport.classList.add("is-dragging");
        return;
      }

      state.dragging = true;
      state.lastX = event.clientX;
      state.lastY = event.clientY;
      viewport.classList.add("is-dragging");
    });

    viewport.addEventListener("pointermove", function (event) {
      updateCanvasHint(event);

      if (!state.pointers.has(event.pointerId)) {
        return;
      }

      state.pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });

      if (state.pointers.size === 2) {
        var pinchPoints = Array.from(state.pointers.values());
        var pinchDistance = Math.hypot(
          pinchPoints[0].x - pinchPoints[1].x,
          pinchPoints[0].y - pinchPoints[1].y
        );
        if (state.pinchDistance > 0) {
          zoomAt(
            state,
            viewport,
            state.pinchScale * pinchDistance / state.pinchDistance,
            (pinchPoints[0].x + pinchPoints[1].x) / 2,
            (pinchPoints[0].y + pinchPoints[1].y) / 2
          );
        }
        render(state);
        return;
      }

      if (!state.dragging) {
        return;
      }

      state.x += event.clientX - state.lastX;
      state.y += event.clientY - state.lastY;
      state.lastX = event.clientX;
      state.lastY = event.clientY;
      render(state);
    });

    function stopDragging(event) {
      state.pointers.delete(event.pointerId);
      if (viewport.hasPointerCapture(event.pointerId)) {
        viewport.releasePointerCapture(event.pointerId);
      }

      if (state.pointers.size > 0) {
        var remaining = state.pointers.values().next().value;
        state.lastX = remaining.x;
        state.lastY = remaining.y;
        state.dragging = true;
        return;
      }

      state.dragging = false;
      viewport.classList.remove("is-dragging");
    }

    viewport.addEventListener("pointerup", stopDragging);
    viewport.addEventListener("pointercancel", stopDragging);
    wrapper.addEventListener("keydown", function (event) {
      if (event.key === "+" || event.key === "=") {
        state.zoomIn.click();
      } else if (event.key === "-" || event.key === "_") {
        state.zoomOut.click();
      } else if (event.key.toLowerCase() === "r") {
        reset.click();
      } else if (event.key.toLowerCase() === "f") {
        fullscreen.click();
      }
    });
  }

  function scan() {
    document.querySelectorAll(".mermaid:not([" + ENHANCED + "])").forEach(enhance);
  }

  function scanAfterMermaidRenders() {
    var attempts = 0;
    var retry = window.setInterval(function () {
      scan();
      attempts += 1;

      if (attempts >= 40 || !document.querySelector(".mermaid:not([" + ENHANCED + "])")) {
        window.clearInterval(retry);
      }
    }, 250);
  }

  function start() {
    scan();
    scanAfterMermaidRenders();
    new MutationObserver(scan).observe(document.body, { childList: true, subtree: true });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
  } else {
    start();
  }
}());
