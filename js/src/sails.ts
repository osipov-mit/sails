import { TypeRegistry } from '@polkadot/types/create';
import { u8aToHex, compactFromU8aLim } from '@polkadot/util';
import { UserMessageSent } from '@gear-js/api';

import { Program, TypeDef, WasmParser } from './parser/index.js';
import { getScaleCodecDef } from './utils/types.js';

const ZERO_ADDRESS = u8aToHex(new Uint8Array(32));

const getPrefixLimitAndOffset = (payload: Uint8Array) => {
  const [offset, limit] = compactFromU8aLim(payload);

  return {
    offset,
    limit,
  };
};

interface SailsServiceFunc {
  args: { name: string; type: any }[];
  returnType: any;
  isQuery: boolean;
  encodePayload: (...args: any[]) => Uint8Array;
  decodePayload: <T>(bytes: string | Uint8Array) => T;
  decodeResult: <T>(result: string | Uint8Array) => T;
}

interface SailsEvent {
  type: any;
  is: (event: UserMessageSent) => boolean;
  decode: (payload: Uint8Array) => any;
}

interface SailsCtorFunc {
  args: { name: string; type: any }[];
  encodePayload: (...args: any[]) => Uint8Array;
  decodePayload: <T>(bytes: string | Uint8Array) => T;
}

export class Sails {
  private _parser: WasmParser;
  private _program: Program;
  private _scaleTypes: Record<string, any>;
  private _registry: TypeRegistry;

  constructor(parser: WasmParser) {
    this._parser = parser;
  }

  /** #### Create new Sails instance */
  static async new() {
    const parser = new WasmParser();
    return new Sails(await parser.init());
  }

  /**
   * ### Parse IDL from string
   * @param idl - IDL string
   */
  parseIdl(idl: string) {
    this._program = this._parser.parse(idl);
    this.generateScaleCodeTypes();
    return this;
  }

  private generateScaleCodeTypes() {
    const scaleTypes: Record<string, any> = {};

    for (const type of this._program.types) {
      scaleTypes[type.name] = getScaleCodecDef(type.def);
    }

    this._registry = new TypeRegistry();
    this._registry.setKnownTypes({ types: scaleTypes });
    this._registry.register(scaleTypes);

    this._scaleTypes = scaleTypes;
  }

  /** #### Scale code types from the parsed IDL */
  get scaleCodecTypes() {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    return this._scaleTypes;
  }

  /** #### Registry with registered types from the parsed IDL */
  get registry() {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    return this._registry;
  }

  /** #### Functions with arguments and return types from the parsed IDL */
  get functions(): Record<string, SailsServiceFunc> {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    const funcs: Record<string, SailsServiceFunc> = {};

    for (const func of this._program.service.funcs) {
      const params = func.params.map((p) => ({ name: p.name, type: getScaleCodecDef(p.def) }));
      const returnType = getScaleCodecDef(func.def);
      funcs[func.name] = {
        args: params,
        returnType,
        isQuery: func.isQuery,
        encodePayload: (...args): Uint8Array => {
          if (args.length !== args.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          const payload = this.registry.createType(`(String, ${params.map((p) => p.type).join(', ')})`, [
            func.name,
            ...args,
          ]);

          return payload.toU8a();
        },
        decodePayload: <T = any>(bytes: Uint8Array | string) => {
          const payload = this.registry.createType(`(String, ${params.map((p) => p.type).join(', ')})`, bytes);
          return payload[1].toJSON() as T;
        },
        decodeResult: <T = any>(result: Uint8Array | string) => {
          const payload = this.registry.createType(`(String, ${returnType})`, result);
          return payload[1].toJSON() as T;
        },
      };
    }

    return funcs;
  }

  /** #### Program events from the parsed IDL */
  get events(): Record<string, SailsEvent> {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    const events: Record<string, SailsEvent> = {};

    for (const event of this._program.service.events) {
      const t = event.def ? getScaleCodecDef(event.def) : 'Null';
      const typeStr = event.def ? getScaleCodecDef(event.def, true) : 'Null';
      events[event.name] = {
        type: t,
        is: ({ data: { message } }: UserMessageSent) => {
          if (!message.destination.eq(ZERO_ADDRESS)) {
            return false;
          }

          const payload = message.payload.toU8a();
          const { offset, limit } = getPrefixLimitAndOffset(payload);
          const name = this.registry.createType('String', payload.subarray(offset, limit)).toString();

          if (name === event.name) {
            return true;
          } else {
            return false;
          }
        },
        decode: (payload: Uint8Array) => {
          const { limit } = getPrefixLimitAndOffset(payload);
          const data = this.registry.createType(typeStr, payload.subarray(limit));
          return data.toJSON();
        },
      };
    }

    return events;
  }

  /** #### Constructor functions with arguments from the parsed IDL */
  get ctors() {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    const ctor = this._program.ctor;

    if (!ctor) {
      return null;
    }

    const funcs: Record<string, SailsCtorFunc> = {};

    for (const func of ctor.funcs) {
      const params = func.params.map((p) => ({ name: p.name, type: getScaleCodecDef(p.def) }));
      funcs[func.name] = {
        args: params,
        encodePayload: (...args): Uint8Array => {
          if (args.length !== args.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          const payload = this.registry.createType(`(String, ${params.map((p) => p.type).join(', ')})`, [
            func.name,
            ...args,
          ]);

          return payload.toU8a();
        },
        decodePayload: <T = any>(bytes: Uint8Array | string) => {
          const payload = this.registry.createType(`(String, ${params.map((p) => p.type).join(', ')})`, bytes);
          return payload[1].toJSON() as T;
        },
      };
    }

    return funcs;
  }

  /** #### Parsed IDL */
  get program() {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    return this._program;
  }

  /** #### Get type definition by name */
  getTypeDef(name: string): TypeDef {
    return this.program.getTypeByName(name).def;
  }

  getFnName(payload: string | Uint8Array) {
    return this._registry.createType('String', payload).toString();
  }
}
