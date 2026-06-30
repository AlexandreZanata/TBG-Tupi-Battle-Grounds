using System.IO;
using TBG.CharacterSystem.Domain;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    internal static class Mesh3DPresentationScaler
    {
        public static void Apply(Transform root, CharacterDefinitionData data)
        {
            if (root == null || data == null)
            {
                return;
            }

            root.localScale = new Vector3(data.bodyWidthScale, data.heightScale, data.bodyWidthScale);
        }
    }
}
