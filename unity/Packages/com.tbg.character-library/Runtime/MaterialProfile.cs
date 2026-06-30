using System;
using System.Collections.Generic;

namespace TBG.CharacterLibrary
{
    public sealed class MaterialProfile
    {
        public string Id { get; init; }
        public string Faction { get; init; }
        public string ShaderName { get; init; }
        public string GraphPath { get; init; }
        public float Metallic { get; init; }
        public float Smoothness { get; init; }
        public float OcclusionStrength { get; init; }
        public float EmissionStrength { get; init; }
        public IReadOnlyList<string> TextureSlots { get; init; }

        public static MaterialProfile FromRecord(MaterialProfileJsonRecord record)
        {
            if (record == null || string.IsNullOrWhiteSpace(record.id))
            {
                throw new ArgumentException("Material profile id is required.");
            }

            var props = record.properties ?? new MaterialProfilePropertiesRecord();
            return new MaterialProfile
            {
                Id = record.id,
                Faction = record.faction,
                ShaderName = record.shader,
                GraphPath = record.graph,
                Metallic = props._Metallic,
                Smoothness = props._Smoothness,
                OcclusionStrength = props._OcclusionStrength,
                EmissionStrength = props._EmissionStrength,
                TextureSlots = record.textureSlots ?? Array.Empty<string>()
            };
        }
    }
}
