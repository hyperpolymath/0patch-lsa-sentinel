--  SPDX-License-Identifier: MPL-2.0
pragma SPARK_Mode (On);

package body Sentinel.Classifier is

   function Classify (S : Signals) return Verdict is
   begin
      --  1. No lsass-targeted patches outstanding -> the conflict is moot.
      if S.Patches = Patch_Absent then
         return Moot;
      end if;

      --  From here on, patches are Present or Unknown.

      --  2. Genuinely covered: protection is off and the loader made it in,
      --     and we positively know patches exist.
      if S.Protection = Protection_Off
        and then S.Loader = Loader_Loaded
        and then S.Patches = Patch_Present
      then
         return Open_Covered;
      end if;

      --  3. The silent-failure state: we positively know patches exist, and
      --     either LSA Protection is on (which necessarily blocks the loader)
      --     or we have observed the loader being blocked.
      if S.Patches = Patch_Present
        and then
          (S.Protection = Protection_On
           or else S.Loader = Loader_Blocked)
      then
         return Blocked_Inert;
      end if;

      --  4. Not enough positive information to assert any of the above.
      return Indeterminate;
   end Classify;

end Sentinel.Classifier;
