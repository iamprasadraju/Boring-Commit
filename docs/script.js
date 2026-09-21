(function () {
  "use strict";

  var reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  // Header border on scroll
  var nav = document.querySelector(".nav");
  function onScroll() {
    if (!nav) return;
    nav.classList.toggle("scrolled", window.scrollY > 8);
  }
  window.addEventListener("scroll", onScroll, { passive: true });
  onScroll();

  // Reveal on scroll
  var revealEls = document.querySelectorAll(".reveal");
  if ("IntersectionObserver" in window && !reduceMotion) {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          entry.target.classList.add("in");
          io.unobserve(entry.target);
        }
      });
    }, { threshold: 0.12, rootMargin: "0px 0px -40px 0px" });
    revealEls.forEach(function (el) { io.observe(el); });
  } else {
    revealEls.forEach(function (el) { el.classList.add("in"); });
  }

  // Scroll-spy for loop sidenotes: the note nearest the viewport
  // middle goes live. No-JS keeps every note fully visible.
  var notes = document.querySelectorAll(".sidenote");
  if (notes.length && "IntersectionObserver" in window && !reduceMotion) {
    var spy = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          notes.forEach(function (n) { n.classList.remove("live"); });
          entry.target.classList.add("live");
        }
      });
    }, { rootMargin: "-40% 0px -40% 0px", threshold: 0 });
    notes.forEach(function (n) { spy.observe(n); });
    if (notes[0]) notes[0].classList.add("live");
  }

  // Copy helper shared by install buttons and reference copy buttons.
  // On success the button shows a visible tick for 1.4s (plus aria feedback).
  var TICK_SVG = '<svg class="tick" width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="m3 8.5 3.2 3L13 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>';
  function flashTick(btn, on) {
    var src = btn.querySelector("svg:not(.tick)");
    var tick = btn.querySelector("svg.tick");
    if (on) {
      if (!tick) btn.insertAdjacentHTML("beforeend", TICK_SVG);
      else tick.style.display = "";
      if (src) src.style.display = "none";
      btn.classList.add("copied");
    } else {
      btn.classList.remove("copied");
      var t = btn.querySelector("svg.tick");
      if (t) t.style.display = "none";
      if (src) src.style.display = "";
    }
  }
  function copyText(text, btn, labelEl) {
    function done(ok) {
      if (labelEl && labelEl.classList.contains("btn-label")) {
        var original = labelEl.getAttribute("data-original") || labelEl.textContent;
        labelEl.setAttribute("data-original", original);
        labelEl.textContent = ok ? "Copied to clipboard" : text;
        window.setTimeout(function () { labelEl.textContent = original; }, 1400);
      } else {
        var label = btn.getAttribute("data-original") || btn.getAttribute("aria-label") || "Copy";
        btn.setAttribute("data-original", label);
        btn.setAttribute("aria-label", ok ? "Copied" : label);
        if (ok) flashTick(btn, true);
        window.setTimeout(function () {
          btn.setAttribute("aria-label", label);
          flashTick(btn, false);
        }, 1400);
      }
    }
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(function () { done(true); }, function () { done(false); });
    } else {
      var ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      try {
        document.execCommand("copy");
        done(true);
      } catch (err) {
        done(false);
      }
      document.body.removeChild(ta);
    }
  }

  document.querySelectorAll("[data-copy]:not(.install-cmd)").forEach(function (btn) {
    btn.addEventListener("click", function () {
      copyText(btn.getAttribute("data-copy") || "", btn, btn.querySelector(".btn-label"));
    });
  });

  document.querySelectorAll(".copy-btn").forEach(function (btn) {
    btn.addEventListener("click", function () {
      var card = btn.closest(".ref-card");
      var code = card ? card.querySelector("pre code") : null;
      if (!code) return;
      copyText(code.textContent, btn, btn);
    });
  });
  document.querySelectorAll(".install-cmd[data-copy]").forEach(function (btn) {
    btn.addEventListener("click", function () {
      copyText(btn.getAttribute("data-copy") || "", btn, btn);
    });
  });
  // OS badge cycle: windows -> apple -> linux every 3s
  var osBadges = document.querySelectorAll(".os-badge .os");
  if (osBadges.length > 1 && !reduceMotion) {
    var osIdx = 0;
    var osTimer = null;
    var cycleOs = function () {
      osBadges[osIdx].classList.remove("active");
      osIdx = (osIdx + 1) % osBadges.length;
      osBadges[osIdx].classList.add("active");
    };
    var startOs = function () {
      if (!osTimer) osTimer = window.setInterval(cycleOs, 3000);
    };
    var stopOs = function () {
      if (osTimer) { window.clearInterval(osTimer); osTimer = null; }
    };
    document.addEventListener("visibilitychange", function () {
      if (document.hidden) stopOs(); else startOs();
    });
    startOs();
  }
  // OS tabs with platform auto-detect (mac fallback, works without JS too)
  var tabs = document.querySelectorAll('.os-tab[data-tab]');
  function selectOsTab(name, focusTab) {
    tabs.forEach(function (tab) {
      var on = tab.getAttribute("data-tab") === name;
      tab.setAttribute("aria-selected", on ? "true" : "false");
      tab.tabIndex = on ? 0 : -1;
      var panel = document.querySelector('.os-panel[data-panel="' + tab.getAttribute("data-tab") + '"]');
      if (panel) panel.hidden = !on;
      if (on && focusTab) tab.focus();
    });
  }
  if (tabs.length) {
    var plat = "";
    if (navigator.userAgentData && navigator.userAgentData.platform) {
      plat = navigator.userAgentData.platform;
    } else if (navigator.platform) {
      plat = navigator.platform;
    }
    plat = plat.toLowerCase();
    var detected = /win/.test(plat) ? "windows" : (/linux/.test(plat) ? "linux" : "mac");
    if (document.querySelector('.os-tab[data-tab="' + detected + '"]')) {
      selectOsTab(detected, false);
    }
    tabs.forEach(function (tab) {
      tab.addEventListener("click", function () {
        selectOsTab(tab.getAttribute("data-tab"), false);
      });
      tab.addEventListener("keydown", function (event) {
        var names = ["mac", "linux", "windows"];
        var i = names.indexOf(tab.getAttribute("data-tab"));
        var next = null;
        if (event.key === "ArrowRight") next = names[(i + 1) % names.length];
        if (event.key === "ArrowLeft") next = names[(i + names.length - 1) % names.length];
        if (next) {
          event.preventDefault();
          selectOsTab(next, true);
        }
      });
    });
  }
})();
