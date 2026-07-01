using UnityEngine;

namespace TBG.CharacterLibrary
{
    internal static class Mesh3DMaterialApplier
    {
        public static void Apply(
            GameObject root,
            LoadedCharacterDefinition definition,
            MaterialProfileLoader materialLoader)
        {
            if (root == null || materialLoader == null)
            {
                return;
            }

            if (string.IsNullOrWhiteSpace(definition.MaterialProfileId) || definition.Textures == null)
            {
                return;
            }

            var profile = materialLoader.Load(definition.MaterialProfileId);
            var material = materialLoader.CreateMaterial(profile);
            BindTextures(material, definition.Textures);

            foreach (var renderer in root.GetComponentsInChildren<Renderer>())
            {
                renderer.sharedMaterial = material;
            }
        }

        private static void BindTextures(Material material, CharacterTexturePaths textures)
        {
            AssignTexture(material, "_MainTex", textures.Albedo);
            AssignTexture(material, "_BumpMap", textures.Normal, normalMap: true);
            AssignTexture(material, "_ORMMap", textures.Orm);
            if (!string.IsNullOrWhiteSpace(textures.Emissive))
            {
                AssignTexture(material, "_EmissionMap", textures.Emissive);
            }
        }

        private static void AssignTexture(
            Material material,
            string property,
            string repoRelativePath,
            bool normalMap = false)
        {
            if (string.IsNullOrWhiteSpace(repoRelativePath))
            {
                return;
            }

            var texture = Mesh3DTextureLoader.LoadPng(repoRelativePath);
            material.SetTexture(property, texture);
            if (normalMap)
            {
                material.EnableKeyword("_NORMALMAP");
            }
        }
    }
}
