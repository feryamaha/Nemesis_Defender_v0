/* ============================================================
   lang.js — troca de idioma PT/EN do onboarding.
   Mesmo padrão do index.html da raiz: elementos com data-pt e
   data-en têm o innerHTML trocado; imagens com data-alt-en têm
   o alt trocado. Preferência persistida em localStorage para
   valer entre as páginas do material.
   ============================================================ */
(function () {
  "use strict";
  var btnPT = document.getElementById("lang-pt");
  var btnEN = document.getElementById("lang-en");
  if (!btnPT || !btnEN) return;

  var nodes = Array.prototype.slice.call(document.querySelectorAll("[data-pt][data-en]"));
  var imgs = Array.prototype.slice.call(document.querySelectorAll("img[data-alt-en]"));

  function applyLang(lang) {
    nodes.forEach(function (n) {
      var val = n.getAttribute(lang === "en" ? "data-en" : "data-pt");
      if (val != null) n.innerHTML = val;
    });
    imgs.forEach(function (i) {
      if (!i.getAttribute("data-alt-pt")) i.setAttribute("data-alt-pt", i.getAttribute("alt") || "");
      i.setAttribute("alt", i.getAttribute(lang === "en" ? "data-alt-en" : "data-alt-pt"));
    });
    btnPT.classList.toggle("active", lang !== "en");
    btnEN.classList.toggle("active", lang === "en");
    document.documentElement.lang = lang === "en" ? "en" : "pt-BR";
    try { localStorage.setItem("nemesis-lang", lang); } catch (e) { /* offline/file: ok */ }
  }

  btnPT.addEventListener("click", function () { applyLang("pt"); });
  btnEN.addEventListener("click", function () { applyLang("en"); });

  var saved = "pt";
  try { saved = localStorage.getItem("nemesis-lang") || "pt"; } catch (e) { /* padrão pt */ }
  if (saved === "en") applyLang("en");
})();
