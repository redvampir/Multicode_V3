import { describe, it, expect } from 'vitest';
import {
  Block,
  registerBlock,
  unregisterBlock,
  createBlock,
  NumberLiteralBlock,
  StringLiteralBlock,
  BooleanLiteralBlock,
  NullLiteralBlock,
  ArrayNewBlock,
  ArrayGetBlock,
  ArraySetBlock,
  MapNewBlock,
  MapGetBlock,
  MapSetBlock,
  VariableGetBlock,
  VariableSetBlock,
  StructBlock,
  SequenceBlock,
  SwitchBlock,
  TryBlock,
  FunctionDefineBlock,
  FunctionCallBlock,
  ReturnBlock,
  LogBlock,
  ForLoopBlock,
  WhileLoopBlock,
  ForEachLoopBlock,
  BreakBlock,
  ContinueBlock,
  AddBlock,
  SubtractBlock,
  MultiplyBlock,
  DivideBlock,
  ModuloBlock,
  OpConcatBlock,
  OpPlusMicroBlock,
  OpMultiplyMicroBlock,
  OpIncBlock,
  OpDecBlock,
  OpAndBlock,
  OpOrBlock,
  OpNotBlock,
  OpEqualBlock,
  OpNotEqualBlock,
  OpGreaterBlock,
  OpGreaterEqualBlock,
  OpLessBlock,
  OpLessEqualBlock
} from './blocks.js';
import { getTheme } from './theme.ts';

describe('block utilities', () => {
  it('checks point containment and center', () => {
    const b = new Block('1', 0, 0, 10, 10, 'test');
    expect(b.contains(5,5)).toBe(true);
    expect(b.center()).toEqual({ x: 5, y: 5 });
  });

  it('registers and creates custom block', () => {
    class Custom extends Block {
      constructor(id, x, y) { super(id, x, y, 10, 10, 'c'); }
    }
    registerBlock('custom', Custom);
    const b = createBlock('custom', '2', 0, 0, 'c');
    expect(b).toBeInstanceOf(Custom);
  });

  it('unregisters a block type', () => {
    class Custom extends Block {
      constructor(id, x, y) { super(id, x, y, 10, 10, 'c'); }
    }
    registerBlock('temp', Custom);
    unregisterBlock('temp');
    const b = createBlock('temp', '3', 0, 0, 'c');
    expect(b).toBeInstanceOf(Block);
  });

  it('provides built-in literal blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Literal/Number', NumberLiteralBlock],
      ['Literal/String', StringLiteralBlock],
      ['Literal/Boolean', BooleanLiteralBlock],
      ['Literal/Null', NullLiteralBlock]
    ];
    for (const [kind, Ctor] of cases) {
      const b = createBlock(kind, 'lit', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.w).toBe(120);
      expect(b.h).toBe(50);
      expect(b.ports).toEqual([{ id: 'out', kind: 'data', dir: 'out' }]);
      expect(b.color).toBe(theme.blockKinds.Literal);
    }
  });

  it('provides array blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Array/New', ArrayNewBlock, ArrayNewBlock.ports],
      ['Array/Get', ArrayGetBlock, ArrayGetBlock.ports],
      ['Array/Set', ArraySetBlock, ArraySetBlock.ports]
    ];
    for (const [kind, Ctor, ports] of cases) {
      const b = createBlock(kind, 'arr', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(ports);
      expect(b.color).toBe(theme.blockKinds.Array);
    }
  });

  it('provides map blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Map/New', MapNewBlock, MapNewBlock.ports],
      ['Map/Get', MapGetBlock, MapGetBlock.ports],
      ['Map/Set', MapSetBlock, MapSetBlock.ports]
    ];
    for (const [kind, Ctor, ports] of cases) {
      const b = createBlock(kind, 'map', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(ports);
      expect(b.color).toBe(theme.blockKinds.Map);
    }
  });

  it('provides operator blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Operator/Add', AddBlock, '+'],
      ['Operator/Subtract', SubtractBlock, '-'],
      ['Operator/Multiply', MultiplyBlock, '*'],
      ['Operator/Divide', DivideBlock, '/'],
      ['Operator/Modulo', ModuloBlock, '%'],
      ['Operator/Concat', OpConcatBlock, '++']
    ];
    for (const [kind, Ctor, label] of cases) {
      const b = createBlock(kind, 'op', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(Ctor.ports);
      expect(b.label).toBe(label);
      expect(b.color).toBe(theme.blockKinds.Operator || theme.blockFill);
    }
  });

  it('provides micro operator blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Op/+', OpPlusMicroBlock, '+'],
      ['Op/*', OpMultiplyMicroBlock, '*'],
      ['Op/Inc', OpIncBlock, '++'],
      ['Op/Dec', OpDecBlock, '--']
    ];
    for (const [kind, Ctor, label] of cases) {
      const b = createBlock(kind, 'mop', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.w).toBe(56);
      expect(b.h).toBe(28);
      expect(b.ports).toEqual(Ctor.ports);
      expect(b.label).toBe(label);
      expect(b.color).toBe(theme.blockKinds.Operator || theme.blockFill);
    }
  });

  it('provides logic operator blocks', () => {
    const theme = getTheme();
    const cases = [
      ['OpLogic/And', OpAndBlock, '&&'],
      ['OpLogic/Or', OpOrBlock, '||'],
      ['OpLogic/Not', OpNotBlock, '!']
    ];
    for (const [kind, Ctor, label] of cases) {
      const b = createBlock(kind, 'logic', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(Ctor.ports);
      expect(b.label).toBe(label);
      expect(b.color).toBe(theme.blockKinds.OpLogic || theme.blockFill);
    }
  });

  it('provides comparison operator blocks', () => {
    const theme = getTheme();
    const cases = [
      ['OpComparison/Equal', OpEqualBlock, '=='],
      ['OpComparison/NotEqual', OpNotEqualBlock, '!='],
      ['OpComparison/Greater', OpGreaterBlock, '>'],
      ['OpComparison/GreaterEqual', OpGreaterEqualBlock, '>='],
      ['OpComparison/Less', OpLessBlock, '<'],
      ['OpComparison/LessEqual', OpLessEqualBlock, '<=']
    ];
    for (const [kind, Ctor, label] of cases) {
      const b = createBlock(kind, 'cmp', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(Ctor.ports);
      expect(b.label).toBe(label);
      expect(b.color).toBe(theme.blockKinds.OpComparison || theme.blockFill);
    }
  });

  it('provides function blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Function/Define', FunctionDefineBlock],
      ['Function/Call', FunctionCallBlock],
      ['Return', ReturnBlock]
    ];
    for (const [kind, Ctor] of cases) {
      const b = createBlock(kind, 'fn', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(Ctor.ports);
      expect(b.color).toBe(theme.blockKinds.Function);
    }
  });

  it('provides log block with optional exec', () => {
    const theme = getTheme();
    const b = createBlock('Log', 'l1', 0, 0, '');
    expect(b).toBeInstanceOf(LogBlock);
    expect(b.ports).toEqual([{ id: 'data', kind: 'data', dir: 'in', auto: true }]);
    expect(b.color).toBe(theme.blockKinds.Log || theme.blockFill);
    const bExec = createBlock('Log', 'l2', 0, 0, '', undefined, { exec: true });
    expect(bExec.ports).toEqual([
      { id: 'data', kind: 'data', dir: 'in', auto: true },
      { id: 'exec', kind: 'exec', dir: 'in' },
      { id: 'out', kind: 'exec', dir: 'out' }
    ]);
  });

  it('provides variable get/set blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Variable/Get', VariableGetBlock, VariableGetBlock.ports],
      ['Variable/Set', VariableSetBlock, VariableSetBlock.ports]
    ];
    for (const [kind, Ctor, ports] of cases) {
      const b = createBlock(kind, 'var', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(ports);
      expect(b.color).toBe(theme.blockKinds.Variable);
      if (kind === 'Variable/Get') {
        expect(b.w).toBe(56);
        expect(b.h).toBe(28);
      }
    }
  });

  it('provides loop blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Loop/For', ForLoopBlock, ForLoopBlock.ports],
      ['Loop/While', WhileLoopBlock, WhileLoopBlock.ports],
      ['Loop/ForEach', ForEachLoopBlock, ForEachLoopBlock.ports],
      ['Loop/Break', BreakBlock, BreakBlock.ports],
      ['Loop/Continue', ContinueBlock, ContinueBlock.ports]
    ];
    for (const [kind, Ctor, ports] of cases) {
      const b = createBlock(kind, 'loop', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(ports);
      expect(b.color).toBe(theme.blockKinds.Loop || theme.blockFill);
    }
  });

  it('provides struct block', () => {
    const theme = getTheme();
    const b = createBlock('Struct', 's', 0, 0, '');
    expect(b).toBeInstanceOf(StructBlock);
    expect(b.ports).toEqual(StructBlock.ports);
    expect(b.color).toBe(theme.blockKinds.Struct || theme.blockFill);
  });

  it('provides switch block with dynamic cases', () => {
    const theme = getTheme();
    const b = createBlock('Switch', 'sw', 0, 0, '', undefined, { cases: ['a', 'b'] });
    expect(b).toBeInstanceOf(SwitchBlock);
    expect(b.ports).toEqual([
      { id: 'value', kind: 'data', dir: 'in' },
      { id: 'exec', kind: 'exec', dir: 'in' },
      { id: 'case[a]', kind: 'exec', dir: 'out' },
      { id: 'case[b]', kind: 'exec', dir: 'out' },
      { id: 'default', kind: 'exec', dir: 'out' }
    ]);
    expect(b.color).toBe(theme.blockKinds.Switch || theme.blockFill);
  });

  it('provides try block', () => {
    const theme = getTheme();
    const b = createBlock('Try', 't', 0, 0, '');
    expect(b.ports).toEqual([
      { id: 'exec', kind: 'exec', dir: 'in' },
      { id: 'try', kind: 'exec', dir: 'out' },
      { id: 'catch', kind: 'exec', dir: 'out' },
      { id: 'finally', kind: 'exec', dir: 'out' }
    ]);
    expect(b.color).toBe(theme.blockKinds.Try || theme.blockFill);
  });

  it('preserves order of sequence block steps', () => {
    const theme = getTheme();
    const b = createBlock('Sequence', 'seq', 0, 0, '', undefined, { steps: ['a', 'b'] });
    expect(b).toBeInstanceOf(SequenceBlock);
    expect(b.ports).toEqual([
      { id: 'exec[a]', kind: 'exec', dir: 'in' },
      { id: 'exec[b]', kind: 'exec', dir: 'in' },
      { id: 'out', kind: 'exec', dir: 'out' }
    ]);
    expect(b.color).toBe(theme.blockKinds.Sequence || theme.blockFill);
    expect(b.steps).toEqual(['a', 'b']);
  });
});
