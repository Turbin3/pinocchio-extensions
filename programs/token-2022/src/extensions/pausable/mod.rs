pub mod initialize;
pub mod pause;
pub mod resume;

/// Discriminator for `TokenInstruction::PausableExtension`
const PAUSABLE_EXTENSION: u8 = 44;

/// Discriminator for `PausableInstruction::Initialze`
const INITIALIZE: u8 = 0;

/// Discriminator for `PausableInstruction::Pause`
const PAUSE: u8 = 1;

/// Discriminator for `PausableInstruction::Resume`
const RESUME: u8 = 2;
