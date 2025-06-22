{ lib }:
rec {
  filterDisabledAndDropEnable =
    value:
    if lib.isAttrs value then
      if value ? enable && value.enable == false then
        null
      else
        lib.filterAttrs (k: v: v != null && k != "enable") (
          lib.mapAttrs (_: filterDisabledAndDropEnable) value
        )
    else if lib.isList value then
      lib.filter (v: v != null) (map filterDisabledAndDropEnable value)
    else
      value;
}
