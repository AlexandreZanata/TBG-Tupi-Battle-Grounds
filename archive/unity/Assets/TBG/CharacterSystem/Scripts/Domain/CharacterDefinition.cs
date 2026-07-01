using UnityEngine;

namespace TBG.CharacterSystem.Domain
{
    [CreateAssetMenu(fileName = "CharacterDefinition", menuName = "TBG/Character Definition")]
    public class CharacterDefinition : ScriptableObject
    {
        public string id;
        public string characterName;
        public Archetype archetype;
        public Faction faction;
        public float heightScale = 1f;
        public float bodyWidthScale = 1f;
        public float headScale = 1f;
        public float limbLengthScale = 1f;
        public Color primaryColor = Color.white;
        public Color secondaryColor = Color.gray;
        public AccessoryType[] accessories;
        public EvolutionStage evolutionStage;

        public void ApplyData(CharacterDefinitionData data)
        {
            id = data.id;
            characterName = data.characterName;
            archetype = data.archetype;
            faction = data.faction;
            heightScale = data.heightScale;
            bodyWidthScale = data.bodyWidthScale;
            headScale = data.headScale;
            limbLengthScale = data.limbLengthScale;
            primaryColor = ColorUtility.TryParseHtmlString(data.primaryColor, out var primary)
                ? primary
                : Color.white;
            secondaryColor = ColorUtility.TryParseHtmlString(data.secondaryColor, out var secondary)
                ? secondary
                : Color.gray;
            accessories = data.accessories ?? new AccessoryType[0];
            evolutionStage = (EvolutionStage)data.evolutionStage;
        }
    }
}
