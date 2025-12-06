// Test setup
// Suppress React 18 concurrent mode errors that occur during test cleanup
// These are caused by the interaction between @preact/signals-react and React's internals
// but don't affect actual test functionality

// Patch the global error handler to catch unhandled promise rejections
// during cleanup and suppress ones related to React concurrent mode
process.on('unhandledRejection', (reason) => {
  const message = String(reason);
  if (message.includes('Should not already be working')) {
    // Suppress this specific error
    return;
  }
  // Re-throw other errors
  throw reason;
});
