using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    /// <summary>
    /// Adds a two-level LODGroup: LOD0 full game mesh, LOD1 static mesh at ~50% triangle budget.
    /// </summary>
    public static class CharacterLodGroup
    {
        private const float Lod0ScreenHeight = 0.35f;
        private const float Lod1ScreenHeight = 0.12f;

        public static void Attach(GameObject root, string unitId)
        {
            if (root == null || string.IsNullOrWhiteSpace(unitId))
            {
                return;
            }

            var lod1Path = Path.Combine(
                CharacterPaths.GetRepoRoot(),
                "content/characters/meshes/game/lod1",
                unitId + ".glb");
            if (!File.Exists(lod1Path))
            {
                return;
            }

            var lod1Root = new GameObject("LOD1");
            lod1Root.transform.SetParent(root.transform, false);
            GltfMeshLoader.Instantiate(lod1Path, lod1Root.transform);

            var lod0Renderers = root.GetComponentsInChildren<Renderer>();
            var lod1Renderers = lod1Root.GetComponentsInChildren<Renderer>();
            if (lod0Renderers.Length == 0 || lod1Renderers.Length == 0)
            {
                Object.Destroy(lod1Root);
                return;
            }

            var group = root.GetComponent<LODGroup>() ?? root.AddComponent<LODGroup>();
            group.SetLODs(new[]
            {
                new LOD(Lod0ScreenHeight, lod0Renderers),
                new LOD(Lod1ScreenHeight, lod1Renderers),
            });
            group.RecalculateBounds();
        }
    }
}
