using System.Collections.Generic;

namespace TBG.CharacterLibrary
{
    /// <summary>Runtime bone collider layout for a body class. Phase 04 implements attachment.</summary>
    public interface IHitboxProfile
    {
        string ProfileId { get; }

        IReadOnlyList<BoneColliderDefinition> Colliders { get; }
    }
}
