using UnityEngine;

namespace TBG.CharacterSystem.Domain
{
    public static class EvolutionVisualModifiers
    {
        public static float GetScaleMultiplier(EvolutionStage stage)
        {
            switch (stage)
            {
                case EvolutionStage.First: return 1.1f;
                case EvolutionStage.Second: return 1.2f;
                default: return 1f;
            }
        }

        public static bool ShouldSaturateColors(EvolutionStage stage)
        {
            return stage == EvolutionStage.Second;
        }

        public static Color ApplySaturation(Color color, EvolutionStage stage)
        {
            if (!ShouldSaturateColors(stage))
            {
                return color;
            }

            var hsb = Color.RGBToHSV(color, out var h, out var s, out var v);
            return Color.HSVToRGB(h, Mathf.Min(1f, s * 1.25f), Mathf.Min(1f, v * 1.1f));
        }
    }
}
