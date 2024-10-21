//------------------------------------------------------------------------------
// <auto-generated>
//     This code was generated by a tool.
//
//     Changes to this file may cause incorrect behavior and will be lost if
//     the code is regenerated.
// </auto-generated>
//------------------------------------------------------------------------------

using Substrate.NetApi.Attributes;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Metadata.Base;
using System.Collections.Generic;


namespace Substrate.Gear.Api.Generated.Model.finality_grandpa
{
    
    
    /// <summary>
    /// >> 84 - Composite[finality_grandpa.EquivocationT1]
    /// </summary>
    [SubstrateNodeType(TypeDefEnum.Composite)]
    public sealed class EquivocationT1 : BaseType
    {
        
        /// <summary>
        /// >> round_number
        /// </summary>
        public Substrate.NetApi.Model.Types.Primitive.U64 RoundNumber { get; set; }
        /// <summary>
        /// >> identity
        /// </summary>
        public Substrate.Gear.Api.Generated.Model.sp_consensus_grandpa.app.Public Identity { get; set; }
        /// <summary>
        /// >> first
        /// </summary>
        public Substrate.NetApi.Model.Types.Base.BaseTuple<Substrate.Gear.Api.Generated.Model.finality_grandpa.Prevote, Substrate.Gear.Api.Generated.Model.sp_consensus_grandpa.app.Signature> First { get; set; }
        /// <summary>
        /// >> second
        /// </summary>
        public Substrate.NetApi.Model.Types.Base.BaseTuple<Substrate.Gear.Api.Generated.Model.finality_grandpa.Prevote, Substrate.Gear.Api.Generated.Model.sp_consensus_grandpa.app.Signature> Second { get; set; }
        
        /// <inheritdoc/>
        public override string TypeName()
        {
            return "EquivocationT1";
        }
        
        /// <inheritdoc/>
        public override byte[] Encode()
        {
            var result = new List<byte>();
            result.AddRange(RoundNumber.Encode());
            result.AddRange(Identity.Encode());
            result.AddRange(First.Encode());
            result.AddRange(Second.Encode());
            return result.ToArray();
        }
        
        /// <inheritdoc/>
        public override void Decode(byte[] byteArray, ref int p)
        {
            var start = p;
            RoundNumber = new Substrate.NetApi.Model.Types.Primitive.U64();
            RoundNumber.Decode(byteArray, ref p);
            Identity = new Substrate.Gear.Api.Generated.Model.sp_consensus_grandpa.app.Public();
            Identity.Decode(byteArray, ref p);
            First = new Substrate.NetApi.Model.Types.Base.BaseTuple<Substrate.Gear.Api.Generated.Model.finality_grandpa.Prevote, Substrate.Gear.Api.Generated.Model.sp_consensus_grandpa.app.Signature>();
            First.Decode(byteArray, ref p);
            Second = new Substrate.NetApi.Model.Types.Base.BaseTuple<Substrate.Gear.Api.Generated.Model.finality_grandpa.Prevote, Substrate.Gear.Api.Generated.Model.sp_consensus_grandpa.app.Signature>();
            Second.Decode(byteArray, ref p);
            var bytesLength = p - start;
            TypeSize = bytesLength;
            Bytes = new byte[bytesLength];
            global::System.Array.Copy(byteArray, start, Bytes, 0, bytesLength);
        }
    }
}