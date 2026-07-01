using System.Collections.Generic;
using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    /// <summary>
    /// Tracks animation set ids loaded per body class so units share clip library metadata.
    /// Clips remain embedded in each game GLB for glTFast; AnimatorControllers are shared per body class.
    /// </summary>
    public static class BodyClassClipRegistry
    {
        private static readonly Dictionary<string, string> RegisteredSets = new();

        public static void Register(string bodyClass, string animationSetId)
        {
            if (string.IsNullOrWhiteSpace(bodyClass) || string.IsNullOrWhiteSpace(animationSetId))
            {
                return;
            }

            if (RegisteredSets.TryGetValue(bodyClass, out var existing) && existing != animationSetId)
            {
                Debug.LogWarning(
                    $"BodyClassClipRegistry: {bodyClass} already uses {existing}; ignoring {animationSetId}");
                return;
            }

            RegisteredSets[bodyClass] = animationSetId;
        }

        public static bool TryGetRegisteredSet(string bodyClass, out string animationSetId)
        {
            return RegisteredSets.TryGetValue(bodyClass, out animationSetId);
        }

        public static string ResolveSharedClipDirectory(string bodyClass)
        {
            if (string.IsNullOrWhiteSpace(bodyClass))
            {
                return null;
            }

            return Path.Combine(
                CharacterPaths.GetRepoRoot(),
                "content/characters/animations/clips",
                bodyClass);
        }

        public static void ClearForTests()
        {
            RegisteredSets.Clear();
        }
    }
}
