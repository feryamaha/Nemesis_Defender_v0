/* ============================================================
   nav.js — comportamentos client-side, 100% offline, sem CDN.
   1) Busca local no índice (filtra .tile por texto)
   2) TOC: gera/realça âncoras das <h2> da página
   3) Realce mínimo de sintaxe em <code class="rust"> / <code class="bash">
   ============================================================ */
(function () {
  "use strict";

  /* ---------- 1. Busca local (apenas no index) ---------- */
  var search = document.getElementById("search");
  if (search) {
    var tiles = Array.prototype.slice.call(document.querySelectorAll(".tile"));
    var counter = document.getElementById("search-count");
    var domains = Array.prototype.slice.call(document.querySelectorAll(".domain"));
    var apply = function () {
      var q = search.value.trim().toLowerCase();
      var shown = 0;
      tiles.forEach(function (t) {
        var hit = q === "" || t.textContent.toLowerCase().indexOf(q) !== -1;
        t.hidden = !hit;
        if (hit) shown++;
      });
      // esconde domínios sem resultados
      domains.forEach(function (d) {
        var any = d.querySelectorAll(".tile:not([hidden])").length > 0;
        d.style.display = any ? "" : "none";
      });
      if (counter) counter.textContent = q === "" ? "" : shown + " página(s)";
    };
    search.addEventListener("input", apply);
    // atalho "/" foca a busca
    document.addEventListener("keydown", function (e) {
      if (e.key === "/" && document.activeElement !== search) {
        e.preventDefault();
        search.focus();
      }
    });
  }

  /* ---------- 2. TOC automático ---------- */
  var tocHost = document.getElementById("toc-list");
  if (tocHost) {
    var heads = Array.prototype.slice.call(document.querySelectorAll("main h2"));
    heads.forEach(function (h, i) {
      if (!h.id) h.id = "sec-" + i;
      var a = document.createElement("a");
      a.href = "#" + h.id;
      a.textContent = h.textContent;
      tocHost.appendChild(a);
    });
    var links = Array.prototype.slice.call(tocHost.querySelectorAll("a"));
    var onScroll = function () {
      var pos = window.scrollY + 120;
      var current = null;
      heads.forEach(function (h) { if (h.offsetTop <= pos) current = h.id; });
      links.forEach(function (l) {
        l.classList.toggle("active", l.getAttribute("href") === "#" + current);
      });
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
  }

  /* ---------- 3. Realce de sintaxe mínimo ---------- */
  var KW = ("fn let mut const pub use mod match if else for while loop return impl struct enum " +
    "trait self Self as in ref Some None Ok Err Option Result Vec String true false static where " +
    "unsafe async await move dyn type").split(" ");
  var kwSet = {};
  KW.forEach(function (k) { kwSet[k] = 1; });

  function esc(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function highlight(raw) {
    // tokeniza linha a linha preservando comentários e strings
    var out = "";
    var lines = raw.split("\n");
    for (var li = 0; li < lines.length; li++) {
      var line = lines[li];
      var i = 0, buf = "";
      var flush = function () {
        if (!buf) return;
        // separa palavras para colorir keywords/fns/nums
        buf.replace(/([A-Za-z_][A-Za-z0-9_]*|\d+\.?\d*|[^A-Za-z0-9_]+)/g, function (tok) {
          if (/^\d/.test(tok)) out += '<span class="tok-num">' + esc(tok) + "</span>";
          else if (kwSet[tok]) out += '<span class="tok-kw">' + esc(tok) + "</span>";
          else out += esc(tok);
          return tok;
        });
        buf = "";
      };
      while (i < line.length) {
        var two = line.substr(i, 2);
        var ch = line[i];
        // comentário de linha
        if (two === "//" || ch === "#") {
          flush();
          out += '<span class="tok-com">' + esc(line.slice(i)) + "</span>";
          i = line.length;
          break;
        }
        // string
        if (ch === '"' || ch === "'" || ch === "`") {
          flush();
          var q = ch, j = i + 1, s = ch;
          while (j < line.length) {
            s += line[j];
            if (line[j] === "\\") { if (j + 1 < line.length) { s += line[j + 1]; j += 2; continue; } }
            if (line[j] === q) { j++; break; }
            j++;
          }
          out += '<span class="tok-str">' + esc(s) + "</span>";
          i = j;
          continue;
        }
        buf += ch;
        i++;
      }
      flush();
      if (li < lines.length - 1) out += "\n";
    }
    return out;
  }

  var blocks = document.querySelectorAll("pre > code.rust, pre > code.bash, pre > code.hl");
  Array.prototype.forEach.call(blocks, function (c) {
    c.innerHTML = highlight(c.textContent);
  });
})();
