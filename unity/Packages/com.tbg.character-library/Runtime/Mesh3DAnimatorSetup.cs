using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

#if UNITY_EDITOR
using UnityEditor;
#endif

namespace TBG.CharacterLibrary
{
    internal static class Mesh3DAnimatorSetup
    {
        public static Animator Attach(
            GameObject root,
            LoadedCharacterDefinition definition,
            BehaviorProfile behaviorProfile)
        {
            if (root == null)
            {
                return null;
            }

            var animator = root.GetComponentInChildren<Animator>();
            if (animator == null)
            {
                animator = root.AddComponent<Animator>();
            }

            var controller = ResolveController(definition, behaviorProfile);
            if (controller != null)
            {
                animator.runtimeAnimatorController = controller;
            }

            return animator;
        }

        private static RuntimeAnimatorController ResolveController(
            LoadedCharacterDefinition definition,
            BehaviorProfile behaviorProfile)
        {
#if UNITY_EDITOR
            var candidates = new[]
            {
                definition.AnimatorControllerPath,
                behaviorProfile?.AnimatorControllerPath,
                BuildPackageControllerPath(behaviorProfile?.BodyClass),
            };

            foreach (var candidate in candidates)
            {
                var controller = LoadController(candidate);
                if (controller != null)
                {
                    return controller;
                }
            }
#endif
            return null;
        }

#if UNITY_EDITOR
        private static string BuildPackageControllerPath(string bodyClass)
        {
            if (string.IsNullOrWhiteSpace(bodyClass))
            {
                return null;
            }

            return $"Packages/com.tbg.character-library/Animations/{bodyClass}.controller";
        }

        private static RuntimeAnimatorController LoadController(string path)
        {
            if (string.IsNullOrWhiteSpace(path))
            {
                return null;
            }

            if (path.StartsWith("Packages/", System.StringComparison.Ordinal))
            {
                return AssetDatabase.LoadAssetAtPath<RuntimeAnimatorController>(path);
            }

            var absolutePath = System.IO.Path.GetFullPath(
                System.IO.Path.Combine(CharacterPaths.GetRepoRoot(), path));
            var assetPath = ToAssetPath(absolutePath);
            return string.IsNullOrWhiteSpace(assetPath)
                ? null
                : AssetDatabase.LoadAssetAtPath<RuntimeAnimatorController>(assetPath);
        }

        private static string ToAssetPath(string absolutePath)
        {
            var normalized = absolutePath.Replace('\\', '/');
            var dataPath = Application.dataPath.Replace('\\', '/');
            var packagesRoot = System.IO.Path.GetFullPath(
                System.IO.Path.Combine(dataPath, "..", "Packages")).Replace('\\', '/');

            if (normalized.StartsWith(dataPath, System.StringComparison.OrdinalIgnoreCase))
            {
                return "Assets" + normalized.Substring(dataPath.Length);
            }

            if (normalized.StartsWith(packagesRoot, System.StringComparison.OrdinalIgnoreCase))
            {
                return "Packages" + normalized.Substring(packagesRoot.Length);
            }

            return null;
        }
#endif
    }
}
