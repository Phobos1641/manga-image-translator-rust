// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="index.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="quick-start.html"><strong aria-hidden="true">2.</strong> Quick Start</a></li><li class="chapter-item expanded "><a href="installation.html"><strong aria-hidden="true">3.</strong> Installation</a></li><li class="chapter-item expanded "><a href="usage.html"><strong aria-hidden="true">4.</strong> Usage</a></li><li class="chapter-item expanded "><a href="python-renderer.html"><strong aria-hidden="true">5.</strong> Python Renderer Usage</a></li><li class="chapter-item expanded "><div><strong aria-hidden="true">6.</strong> Hyperlinks to original models</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="modules.html"><strong aria-hidden="true">6.1.</strong> Modules</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="modules/detectors.html"><strong aria-hidden="true">6.1.1.</strong> Detectors</a></li><li class="chapter-item expanded "><a href="modules/ocrs.html"><strong aria-hidden="true">6.1.2.</strong> OCRs</a></li><li class="chapter-item expanded "><a href="modules/upscalers.html"><strong aria-hidden="true">6.1.3.</strong> Upscalers</a></li><li class="chapter-item expanded "><a href="modules/inpainters.html"><strong aria-hidden="true">6.1.4.</strong> Inpainters</a></li><li class="chapter-item expanded "><a href="modules/translators.html"><strong aria-hidden="true">6.1.5.</strong> Translators</a></li></ol></li><li class="chapter-item expanded "><a href="dev/cpp-deps.html"><strong aria-hidden="true">6.2.</strong> CPP Dependencies</a></li></ol></li><li class="chapter-item expanded "><div><strong aria-hidden="true">7.</strong> Developer Docs</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="roadmap.html"><strong aria-hidden="true">7.1.</strong> Roadmap</a></li><li class="chapter-item expanded "><a href="dev/build.html"><strong aria-hidden="true">7.2.</strong> Build from Source</a></li><li class="chapter-item expanded "><a href="dev/deploy.html"><strong aria-hidden="true">7.3.</strong> Deploy</a></li><li class="chapter-item expanded "><a href="dev/binary-format.html"><strong aria-hidden="true">7.4.</strong> Binary Data Structure v1</a></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
