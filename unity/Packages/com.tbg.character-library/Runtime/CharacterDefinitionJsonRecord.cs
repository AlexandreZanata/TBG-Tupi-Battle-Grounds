using System;
using System.Collections.Generic;
using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    [Serializable]
    public sealed class GameLodRecord
    {
        public int triTarget = 8000;
        public int textureSize = 1024;
    }

    [Serializable]
    public sealed class CharacterTexturesRecord
    {
        public string albedo;
        public string normal;
        public string orm;
        public string emissive;
    }

    [Serializable]
    public sealed class AssemblyRecipeRecord
    {
        public string unitId;
        public string baseId;
        public string[] parts;
        public string materialId;
    }

    [Serializable]
    public sealed class CharacterDefinitionJsonRecord
    {
        public string id;
        public string characterName;
        public string archetype;
        public string faction;
        public string assetMode;
        public string mesh;
        public string meshShowcase;
        public string animationSet;
        public GameLodRecord gameLod;
        public CharacterTexturesRecord textures;
        public string materialProfile;
        public string behaviorProfile;
        public string animatorController;
        public AssemblyRecipeRecord assemblyRecipe;
        public float heightScale = 1f;
        public float bodyWidthScale = 1f;
        public float headScale = 1f;
        public float limbLengthScale = 1f;
        public string primaryColor = "#FFFFFF";
        public string secondaryColor = "#CCCCCC";
        public string[] accessories;
        public int evolutionStage;
    }
}
