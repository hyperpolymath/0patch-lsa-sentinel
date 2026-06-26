--  SPDX-License-Identifier: MPL-2.0
--  Root package for the 0patch-lsa-sentinel verified classifier kernel.
pragma SPARK_Mode (On);

package Sentinel is
   --  Pure root namespace. All logic lives in child units:
   --    * Sentinel.Classifier  -- the verified decision kernel
   --    * Sentinel.Abi         -- the C ABI export consumed by Rust/Zig
end Sentinel;
