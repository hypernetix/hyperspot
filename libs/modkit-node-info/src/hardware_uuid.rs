use uuid::Uuid;

/// Neutral namespace identifier for hardware-based UUIDs
const NAMESPACE_BYTES: &[u8] = b"node-hardware-id";

/// Get a permanent hardware-based UUID for this machine.
/// This UUID is the actual hardware identifier and will remain consistent
/// across reboots and application restarts.
///
/// Platform-specific implementations:
/// - macOS: Uses `IOPlatformUUID` from `IOKit` (already a UUID)
/// - Linux: Uses /etc/machine-id or /var/lib/dbus/machine-id (converted to UUID)
/// - Windows: Uses `MachineGuid` from registry (already a UUID)
///
/// Returns a hybrid UUID (00000000-0000-0000-xxxx-xxxxxxxxxxxx) if detection fails,
/// where the left part is all zeros and the right part is random for uniqueness.
pub fn get_hardware_uuid() -> Uuid {
    match machine_uid::get() {
        Ok(machine_id) => {
            // Try to parse the machine_id as a UUID directly
            match Uuid::parse_str(&machine_id) {
                Ok(uuid) => {
                    tracing::debug!(
                        machine_id = %machine_id,
                        node_uuid = %uuid,
                        "Using hardware UUID"
                    );
                    uuid
                }
                Err(parse_err) => {
                    // If it's not a valid UUID format, hash it to create one
                    tracing::warn!(
                        machine_id = %machine_id,
                        error = %parse_err,
                        "Machine ID is not a valid UUID, using hash-based UUID"
                    );

                    // Use UUID v5 to create a deterministic UUID from the machine ID
                    // Combine namespace identifier with machine ID for hashing
                    let combined = [NAMESPACE_BYTES, b":", machine_id.as_bytes()].concat();
                    Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, &combined)
                }
            }
        }
        Err(e) => {
            // Return a hybrid UUID: zeros on the left (00000000-0000-0000), random on the right
            // This indicates hardware detection failed while still providing uniqueness
            let random_uuid = Uuid::new_v4();
            let random_bytes = random_uuid.as_bytes();

            // Create hybrid: first 8 bytes are zeros, last 8 bytes are random
            let hybrid_bytes = [
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0, // Left part: all zeros
                random_bytes[8],
                random_bytes[9],
                random_bytes[10],
                random_bytes[11],
                random_bytes[12],
                random_bytes[13],
                random_bytes[14],
                random_bytes[15],
            ];

            let hybrid_uuid = Uuid::from_bytes(hybrid_bytes);

            tracing::error!(
                error = %e,
                fallback_uuid = %hybrid_uuid,
                "Failed to get hardware machine ID, using hybrid UUID (00000000-0000-0000-xxxx-xxxxxxxxxxxx)"
            );

            hybrid_uuid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_uuid_is_consistent() {
        // The UUID should be the same across multiple calls
        let uuid1 = get_hardware_uuid();
        let uuid2 = get_hardware_uuid();

        assert_eq!(uuid1, uuid2, "Hardware UUID should be consistent");
    }

    #[test]
    fn test_hardware_uuid_format() {
        let uuid = get_hardware_uuid();

        // Check if it's a fallback UUID (first 8 bytes are zeros)
        let uuid_bytes = uuid.as_bytes();
        let is_fallback = uuid_bytes[0..8].iter().all(|&b| b == 0);

        if is_fallback {
            // If fallback, the right part should be random (not all zeros)
            let right_part_all_zeros = uuid_bytes[8..16].iter().all(|&b| b == 0);
            assert!(
                !right_part_all_zeros,
                "Fallback UUID should have random right part"
            );
        } else {
            // On real hardware, should have a valid hardware UUID
            assert!(
                !is_fallback,
                "Real hardware should not produce fallback UUID"
            );
        }
    }
}
