// Search Integration Module
// Bridges the enhanced search functionality with existing ADR search
(function () {
  "use strict";

  // Wait for both search systems to be available
  function initSearchIntegration() {
    if (
      typeof window.CaxtonSearch !== "undefined" &&
      typeof window.ADRSearch !== "undefined"
    ) {
      console.log(
        "Search Integration: Bridging enhanced search with ADR search",
      );

      // Override ADR search to also trigger highlighting
      const originalPerformSearch = window.ADRSearch.performSearch;
      window.ADRSearch.performSearch = function (query) {
        // Call original ADR search
        originalPerformSearch.call(this, query);

        // Also trigger enhanced highlighting
        window.CaxtonSearch.performSearch(query);
      };

      // Override ADR hide results to also clear highlighting
      const originalHideResults = window.ADRSearch.hideSearchResults;
      window.ADRSearch.hideSearchResults = function () {
        // Call original hide function
        originalHideResults.call(this);

        // Also clear highlighting
        window.CaxtonSearch.clearSearch();
      };

      console.log(
        "Search Integration: Successfully integrated enhanced search with ADR search",
      );
    } else {
      // Retry after a short delay if systems aren't ready
      setTimeout(initSearchIntegration, 100);
    }
  }

  // Initialize when DOM is ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initSearchIntegration);
  } else {
    initSearchIntegration();
  }

  // Expose integration status
  window.SearchIntegration = {
    isReady: () =>
      typeof window.CaxtonSearch !== "undefined" &&
      typeof window.ADRSearch !== "undefined",
    getEnhancedSearch: () => window.CaxtonSearch,
    getADRSearch: () => window.ADRSearch,
  };
})();
