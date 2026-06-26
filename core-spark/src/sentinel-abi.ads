--  SPDX-License-Identifier: MPL-2.0
--
--  Sentinel.Abi — the C ABI seam exported to the Rust host (via the Zig
--  `sentinel.h` declarations). Inputs and the result are plain C ints using
--  the canonical encodings documented in Sentinel.Classifier.
--
--  Out-of-range inputs degrade safely to the corresponding *_Unknown value,
--  so a malformed call can never be mistaken for a positive "covered" signal.

pragma SPARK_Mode (On);

with Interfaces.C;

package Sentinel.Abi is

   function Classify_C
     (Protection : Interfaces.C.int;
      Loader     : Interfaces.C.int;
      Patches    : Interfaces.C.int) return Interfaces.C.int
     with
       Export,
       Convention    => C,
       External_Name => "sentinel_classify";

end Sentinel.Abi;
