using TBG.CharacterSystem.Domain;
using UnityEngine;

namespace TBG.CharacterSystem.Runtime
{
    public static class FactionMaterialFactory
    {
        private const string ShaderName = "TBG/FactionSurface";

        public static Material Create(CharacterDefinition definition)
        {
            var shader = Shader.Find(ShaderName);
            if (shader == null)
            {
                shader = Shader.Find("Standard");
            }

            var material = new Material(shader);
            var primary = EvolutionVisualModifiers.ApplySaturation(definition.primaryColor, definition.evolutionStage);
            var secondary = EvolutionVisualModifiers.ApplySaturation(definition.secondaryColor, definition.evolutionStage);
            material.SetColor("_PrimaryColor", primary);
            material.SetColor("_SecondaryColor", secondary);
            material.SetFloat("_FactionMode", (float)definition.faction);
            material.SetFloat("_EvolutionGlow", definition.evolutionStage == EvolutionStage.Second ? 1f : 0f);
            return material;
        }
    }
}
