// UI module placeholder for future pure render functions
// Each step will have its own module with pure render functions that:
// 1. Read DslState (immutable)
// 2. Render UI based on state
// 3. Dispatch actions to OnboardingManager
// 4. NO direct backend calls

// Future modules:
// pub mod client_info;     // Client info step UI
// pub mod kyc_docs;        // KYC documents step UI
// pub mod risk;            // Risk assessment step UI
// pub mod account_setup;   // Account setup step UI
// pub mod review;          // Review step UI
// pub mod complete;        // Completion step UI