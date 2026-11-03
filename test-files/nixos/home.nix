{ lib, inputs, ... }:
{
  home-manager = {
    useGlobalPkgs = true;
    extraSpecialArgs = { inherit inputs; };
    users.user1 = import ./user1.nix;
    users.user2 = import ./user2.nix;
  };
}
