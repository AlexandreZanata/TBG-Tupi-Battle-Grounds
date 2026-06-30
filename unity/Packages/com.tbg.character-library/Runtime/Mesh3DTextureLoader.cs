using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    internal static class Mesh3DTextureLoader
    {
        public static Texture2D LoadPng(string repoRelativePath)
        {
            if (string.IsNullOrWhiteSpace(repoRelativePath))
            {
                return null;
            }

            var absolutePath = Path.GetFullPath(
                Path.Combine(CharacterPaths.GetRepoRoot(), repoRelativePath));
            if (!File.Exists(absolutePath))
            {
                throw new FileNotFoundException("Texture file not found.", absolutePath);
            }

            var bytes = File.ReadAllBytes(absolutePath);
            var texture = new Texture2D(2, 2, TextureFormat.RGBA32, true);
            if (!texture.LoadImage(bytes))
            {
                Object.Destroy(texture);
                throw new InvalidOperationException($"Failed to decode texture: {repoRelativePath}");
            }

            texture.name = Path.GetFileNameWithoutExtension(absolutePath);
            return texture;
        }
    }
}
