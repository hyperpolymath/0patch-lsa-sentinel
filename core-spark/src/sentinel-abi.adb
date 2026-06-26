--  SPDX-License-Identifier: MPL-2.0
pragma SPARK_Mode (On);

with Sentinel.Classifier; use Sentinel.Classifier;

package body Sentinel.Abi is

   function To_Protection (X : Interfaces.C.int) return Lsa_Protection is
     (case X is
         when 1      => Protection_Off,
         when 2      => Protection_On,
         when others => Protection_Unknown);

   function To_Loader (X : Interfaces.C.int) return Loader_Presence is
     (case X is
         when 1      => Loader_Blocked,
         when 2      => Loader_Loaded,
         when others => Loader_Unknown);

   function To_Patches (X : Interfaces.C.int) return Patch_Status is
     (case X is
         when 1      => Patch_Absent,
         when 2      => Patch_Present,
         when others => Patch_Unknown);

   function Encode (V : Verdict) return Interfaces.C.int is
     (case V is
         when Indeterminate => 0,
         when Moot          => 1,
         when Open_Covered  => 2,
         when Blocked_Inert => 3);

   function Classify_C
     (Protection : Interfaces.C.int;
      Loader     : Interfaces.C.int;
      Patches    : Interfaces.C.int) return Interfaces.C.int
   is
      S : constant Signals :=
        (Protection => To_Protection (Protection),
         Loader     => To_Loader (Loader),
         Patches    => To_Patches (Patches));
   begin
      return Encode (Classify (S));
   end Classify_C;

end Sentinel.Abi;
