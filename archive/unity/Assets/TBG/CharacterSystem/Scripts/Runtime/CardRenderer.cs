using System.IO;
using UnityEngine;

namespace TBG.CharacterSystem.Runtime
{
    public class CardRenderer : MonoBehaviour
    {
        [SerializeField] private Camera cardCamera;
        [SerializeField] private int width = 512;
        [SerializeField] private int height = 768;

        public string RenderToFile(GameObject character, string outputPath)
        {
            var renderTexture = new RenderTexture(width, height, 24);
            cardCamera.targetTexture = renderTexture;
            cardCamera.Render();

            var previous = RenderTexture.active;
            RenderTexture.active = renderTexture;
            var texture = new Texture2D(width, height, TextureFormat.RGB24, false);
            texture.ReadPixels(new Rect(0, 0, width, height), 0, 0);
            texture.Apply();
            RenderTexture.active = previous;

            Directory.CreateDirectory(Path.GetDirectoryName(outputPath));
            File.WriteAllBytes(outputPath, texture.EncodeToPNG());
            DestroyImmediate(texture);
            cardCamera.targetTexture = null;
            renderTexture.Release();
            return outputPath;
        }
    }
}
