// Test setup for preact-router-test
import '@testing-library/preact';

// Ensure global mocks
beforeEach(() => {
  // Reset DOM
  document.body.innerHTML = '<div id="app"></div>';
  
  // Reset window location
  window.history.pushState({}, '', '/');
});
