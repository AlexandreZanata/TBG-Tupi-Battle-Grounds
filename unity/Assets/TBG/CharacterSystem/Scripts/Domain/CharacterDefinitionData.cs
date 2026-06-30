using System;
using UnityEngine;

namespace TBG.CharacterSystem.Domain
{
    [Serializable]
    public class CharacterDefinitionData
    {
        public string id;
        public string characterName;
        public Archetype archetype;
        public Faction faction;
        public float heightScale = 1f;
        public float bodyWidthScale = 1f;
        public float headScale = 1f;
        public float limbLengthScale = 1f;
        public string primaryColor = "#FFFFFF";
        public string secondaryColor = "#CCCCCC";
        public AccessoryType[] accessories = Array.Empty<AccessoryType>();
        public int evolutionStage;
    }
}
