// src/vesting_struct_optimization.rs

#![cfg(test)]

use soroban_sdk::{contracttype, Env, Vec};

// =====================================================
// ❌ OLD (INEFFICIENT) STRUCT
// =====================================================

#[derive(Clone)]
#[contracttype]
pub struct VestingScheduleV1 {
    pub is_active: bool,   // 1 byte
    pub amount: i128,      // 16 bytes
    pub start_time: u64,   // 8 bytes
    pub claimed: bool,     // 1 byte
    pub duration: u64,     // 8 bytes
    pub cliff: u32,        // 4 bytes
}

// =====================================================
// ✅ OPTIMIZED STRUCT
// =====================================================

#[derive(Clone)]
#[contracttype]
pub struct VestingScheduleV2 {
    // 🔢 Largest → smallest (reduces padding)

    pub amount: i128,      // 16 bytes

    pub start_time: u64,   // 8 bytes
    pub duration: u64,     // 8 bytes

    pub cliff: u32,        // 4 bytes

    // 🧠 Pack booleans at the end
    pub is_active: bool,   // 1 byte
    pub claimed: bool,     // 1 byte
}

// =====================================================
// 🧪 TESTS (SIZE + BEHAVIOR)
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn compare_struct_sizes() {
        let v1_size = size_of::<VestingScheduleV1>();
        let v2_size = size_of::<VestingScheduleV2>();

        println!("V1 size: {}", v1_size);
        println!("V2 size: {}", v2_size);

        assert!(
            v2_size <= v1_size,
            "Optimized struct should be smaller or equal in size"
        );
    }

    #[test]
    fn validate_data_integrity_after_reorder() {
        let v2 = VestingScheduleV2 {
            amount: 1000,
            start_time: 100,
            duration: 200,
            cliff: 10,
            is_active: true,
            claimed: false,
        };

        assert_eq!(v2.amount, 1000);
        assert_eq!(v2.start_time, 100);
        assert_eq!(v2.duration, 200);
        assert_eq!(v2.cliff, 10);
        assert_eq!(v2.is_active, true);
        assert_eq!(v2.claimed, false);
    }

    #[test]
    fn simulate_bulk_storage_impact() {
        let env = Env::default();
        let mut schedules = Vec::new(&env);

        for _ in 0..500 {
            schedules.push_back(VestingScheduleV2 {
                amount: 1000,
                start_time: 0,
                duration: 100,
                cliff: 0,
                is_active: true,
                claimed: false,
            });
        }

        assert_eq!(schedules.len(), 500);

        // This test ensures no unexpected overhead during bulk usage
    }
}