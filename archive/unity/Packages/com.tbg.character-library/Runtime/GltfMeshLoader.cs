using System;
using System.Threading.Tasks;
using GLTFast;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    internal static class GltfMeshLoader
    {
        public static void Instantiate(string absolutePath, Transform parent)
        {
            var uri = new Uri(absolutePath).AbsoluteUri;
            var gltf = new GltfImport();
            try
            {
                var loaded = RunSync(gltf.Load(uri));
                if (!loaded)
                {
                    throw new InvalidOperationException($"Failed to load glb: {absolutePath}");
                }

                var instantiated = RunSync(gltf.InstantiateMainSceneAsync(parent));
                if (!instantiated)
                {
                    throw new InvalidOperationException($"Failed to instantiate glb scene: {absolutePath}");
                }
            }
            finally
            {
                gltf.Dispose();
            }
        }

        private static T RunSync<T>(Task<T> task) => task.GetAwaiter().GetResult();
    }
}
