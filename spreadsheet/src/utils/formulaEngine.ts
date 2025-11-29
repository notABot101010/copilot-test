import { getCellKey } from '../types/spreadsheet';

export interface CellData {
  value: string;
}

// Convert column letter to index (A=0, B=1, ..., Z=25, AA=26, etc.)
export function columnToIndex(col: string): number {
  let result = 0;
  for (let i = 0; i < col.length; i++) {
    result = result * 26 + (col.charCodeAt(i) - 64);
  }
  return result - 1;
}

// Convert index to column letter
export function indexToColumn(index: number): string {
  let result = '';
  index = index + 1;
  while (index > 0) {
    const remainder = (index - 1) % 26;
    result = String.fromCharCode(65 + remainder) + result;
    index = Math.floor((index - 1) / 26);
  }
  return result;
}

// Parse cell reference like A1, B2, AA10
export function parseCellReference(ref: string): { row: number; col: number } | null {
  const match = ref.match(/^([A-Z]+)(\d+)$/i);
  if (!match) return null;
  const col = columnToIndex(match[1].toUpperCase());
  const row = parseInt(match[2], 10) - 1;
  return { row, col };
}

// Format cell reference from row/col to A1 notation
export function formatCellReference(row: number, col: number): string {
  return `${indexToColumn(col)}${row + 1}`;
}

// Parse range like A1:B5
export function parseRange(range: string): { start: { row: number; col: number }; end: { row: number; col: number } } | null {
  const parts = range.split(':');
  if (parts.length !== 2) return null;
  const start = parseCellReference(parts[0].trim());
  const end = parseCellReference(parts[1].trim());
  if (!start || !end) return null;
  return { start, end };
}

// Get all cells in a range
export function getCellsInRange(
  range: string,
  cells: Record<string, CellData>
): number[] {
  const parsed = parseRange(range);
  if (!parsed) return [];

  const values: number[] = [];
  const minRow = Math.min(parsed.start.row, parsed.end.row);
  const maxRow = Math.max(parsed.start.row, parsed.end.row);
  const minCol = Math.min(parsed.start.col, parsed.end.col);
  const maxCol = Math.max(parsed.start.col, parsed.end.col);

  for (let row = minRow; row <= maxRow; row++) {
    for (let col = minCol; col <= maxCol; col++) {
      const key = getCellKey(row, col);
      const cellValue = cells[key]?.value;
      if (cellValue) {
        const num = parseFloat(cellValue);
        if (!isNaN(num)) {
          values.push(num);
        }
      }
    }
  }
  return values;
}

// Built-in Excel-like functions
export const builtinFunctions: Record<string, (...args: (number | number[])[]) => number | string> = {
  SUM: (...args: (number | number[])[]) => {
    const flat = args.flat();
    return flat.reduce((a, b) => a + (typeof b === 'number' ? b : 0), 0);
  },
  AVERAGE: (...args: (number | number[])[]) => {
    const flat = args.flat().filter((x): x is number => typeof x === 'number');
    if (flat.length === 0) return 0;
    return flat.reduce((a, b) => a + b, 0) / flat.length;
  },
  MIN: (...args: (number | number[])[]) => {
    const flat = args.flat().filter((x): x is number => typeof x === 'number');
    if (flat.length === 0) return 0;
    return Math.min(...flat);
  },
  MAX: (...args: (number | number[])[]) => {
    const flat = args.flat().filter((x): x is number => typeof x === 'number');
    if (flat.length === 0) return 0;
    return Math.max(...flat);
  },
  COUNT: (...args: (number | number[])[]) => {
    return args.flat().filter((x): x is number => typeof x === 'number').length;
  },
  ROUND: (value: number | number[], decimals: number | number[] = 0) => {
    const v = Array.isArray(value) ? value[0] : value;
    const d = Array.isArray(decimals) ? decimals[0] : decimals;
    if (typeof v !== 'number') return 0;
    const factor = Math.pow(10, d || 0);
    return Math.round(v * factor) / factor;
  },
  FLOOR: (value: number | number[]) => {
    const v = Array.isArray(value) ? value[0] : value;
    return typeof v === 'number' ? Math.floor(v) : 0;
  },
  CEIL: (value: number | number[]) => {
    const v = Array.isArray(value) ? value[0] : value;
    return typeof v === 'number' ? Math.ceil(v) : 0;
  },
  ABS: (value: number | number[]) => {
    const v = Array.isArray(value) ? value[0] : value;
    return typeof v === 'number' ? Math.abs(v) : 0;
  },
  SQRT: (value: number | number[]) => {
    const v = Array.isArray(value) ? value[0] : value;
    return typeof v === 'number' && v >= 0 ? Math.sqrt(v) : 0;
  },
  POW: (base: number | number[], exp: number | number[]) => {
    const b = Array.isArray(base) ? base[0] : base;
    const e = Array.isArray(exp) ? exp[0] : exp;
    return typeof b === 'number' && typeof e === 'number' ? Math.pow(b, e) : 0;
  },
  MOD: (a: number | number[], b: number | number[]) => {
    const av = Array.isArray(a) ? a[0] : a;
    const bv = Array.isArray(b) ? b[0] : b;
    return typeof av === 'number' && typeof bv === 'number' && bv !== 0 ? av % bv : 0;
  },
  PI: () => Math.PI,
  IF: (condition: number | number[], trueVal: number | number[], falseVal: number | number[]) => {
    const c = Array.isArray(condition) ? condition[0] : condition;
    const t = Array.isArray(trueVal) ? trueVal[0] : trueVal;
    const f = Array.isArray(falseVal) ? falseVal[0] : falseVal;
    return c ? t : f;
  },
  AND: (...args: (number | number[])[]) => {
    const flat = args.flat();
    return flat.every(x => x) ? 1 : 0;
  },
  OR: (...args: (number | number[])[]) => {
    const flat = args.flat();
    return flat.some(x => x) ? 1 : 0;
  },
  NOT: (value: number | number[]) => {
    const v = Array.isArray(value) ? value[0] : value;
    return v ? 0 : 1;
  },
  CONCAT: (...args: (number | number[])[]) => {
    return args.flat().join('');
  },
  LEN: (value: number | number[]) => {
    const v = Array.isArray(value) ? value[0] : value;
    return String(v).length;
  },
};

// Evaluate a formula
export function evaluateFormula(
  formula: string,
  cells: Record<string, CellData>,
  visitedCells: Set<string> = new Set()
): string {
  if (!formula.startsWith('=')) {
    return formula;
  }

  const expression = formula.substring(1).trim();

  try {
    // Replace cell references with their values
    let processed = expression;

    // Handle range references in function calls (e.g., SUM(A1:B5))
    processed = processed.replace(
      /([A-Z]+\d+):([A-Z]+\d+)/gi,
      (match) => {
        const values = getCellsInRange(match, cells);
        return `[${values.join(',')}]`;
      }
    );

    // Handle single cell references (e.g., A1, B2)
    processed = processed.replace(
      /\b([A-Z]+)(\d+)\b/gi,
      (match) => {
        const ref = parseCellReference(match);
        if (!ref) return '0';
        
        const key = getCellKey(ref.row, ref.col);
        
        // Check for circular reference
        if (visitedCells.has(key)) {
          throw new Error('Circular reference detected');
        }
        
        const cellValue = cells[key]?.value || '0';
        
        // If the referenced cell is also a formula, evaluate it
        if (cellValue.startsWith('=')) {
          const newVisited = new Set(visitedCells);
          newVisited.add(key);
          const result = evaluateFormula(cellValue, cells, newVisited);
          const num = parseFloat(result);
          return isNaN(num) ? '0' : String(num);
        }
        
        const num = parseFloat(cellValue);
        return isNaN(num) ? '0' : String(num);
      }
    );

    // Replace function calls with their results
    for (const [funcName, func] of Object.entries(builtinFunctions)) {
      const regex = new RegExp(`\\b${funcName}\\s*\\(([^)]*)\\)`, 'gi');
      processed = processed.replace(regex, (_, args) => {
        const parsedArgs = parseArguments(args);
        const result = func(...parsedArgs);
        return String(result);
      });
    }

    // Use safe mathematical expression evaluator instead of Function constructor
    const result = safeEvaluate(processed);
    
    if (typeof result === 'number') {
      return isNaN(result) ? '#ERROR' : 
             result === Infinity || result === -Infinity ? '#ERROR' :
             Number.isInteger(result) ? String(result) : result.toFixed(10).replace(/\.?0+$/, '');
    }
    return String(result);
  } catch {
    return '#ERROR';
  }
}

// Token types for the expression parser
type TokenType = 'NUMBER' | 'OPERATOR' | 'LPAREN' | 'RPAREN' | 'END';
interface Token {
  type: TokenType;
  value: string | number;
}

// Tokenize expression into tokens
function tokenize(expr: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  
  while (i < expr.length) {
    const char = expr[i];
    
    // Skip whitespace
    if (/\s/.test(char)) {
      i++;
      continue;
    }
    
    // Numbers (including decimals and negative numbers at start)
    if (/[\d.]/.test(char) || (char === '-' && (tokens.length === 0 || tokens[tokens.length - 1].type === 'OPERATOR' || tokens[tokens.length - 1].type === 'LPAREN'))) {
      let numStr = char;
      i++;
      while (i < expr.length && /[\d.]/.test(expr[i])) {
        numStr += expr[i];
        i++;
      }
      tokens.push({ type: 'NUMBER', value: parseFloat(numStr) });
      continue;
    }
    
    // Operators
    if ('+-*/%'.includes(char)) {
      tokens.push({ type: 'OPERATOR', value: char });
      i++;
      continue;
    }
    
    // Comparison operators
    if ('<>=!'.includes(char)) {
      let op = char;
      i++;
      if (i < expr.length && expr[i] === '=') {
        op += '=';
        i++;
      }
      tokens.push({ type: 'OPERATOR', value: op });
      continue;
    }
    
    // Parentheses
    if (char === '(') {
      tokens.push({ type: 'LPAREN', value: '(' });
      i++;
      continue;
    }
    
    if (char === ')') {
      tokens.push({ type: 'RPAREN', value: ')' });
      i++;
      continue;
    }
    
    // Unknown character - skip
    i++;
  }
  
  tokens.push({ type: 'END', value: '' });
  return tokens;
}

// Simple recursive descent parser for safe expression evaluation
function safeEvaluate(expr: string): number | string {
  const tokens = tokenize(expr);
  let pos = 0;
  
  function peek(): Token {
    return tokens[pos];
  }
  
  function consume(): Token {
    return tokens[pos++];
  }
  
  function parseExpression(): number {
    let left = parseTerm();
    
    while (peek().type === 'OPERATOR' && (peek().value === '+' || peek().value === '-')) {
      const op = consume().value;
      const right = parseTerm();
      if (op === '+') {
        left = left + right;
      } else {
        left = left - right;
      }
    }
    
    return left;
  }
  
  function parseTerm(): number {
    let left = parseComparison();
    
    while (peek().type === 'OPERATOR' && (peek().value === '*' || peek().value === '/' || peek().value === '%')) {
      const op = consume().value;
      const right = parseComparison();
      if (op === '*') {
        left = left * right;
      } else if (op === '/') {
        left = right !== 0 ? left / right : NaN;
      } else {
        left = right !== 0 ? left % right : NaN;
      }
    }
    
    return left;
  }
  
  function parseComparison(): number {
    let left = parseFactor();
    
    while (peek().type === 'OPERATOR' && ['<', '>', '<=', '>=', '==', '!=', '='].includes(peek().value as string)) {
      const op = consume().value;
      const right = parseFactor();
      switch (op) {
        case '<': left = left < right ? 1 : 0; break;
        case '>': left = left > right ? 1 : 0; break;
        case '<=': left = left <= right ? 1 : 0; break;
        case '>=': left = left >= right ? 1 : 0; break;
        case '==':
        case '=': left = left === right ? 1 : 0; break;
        case '!=': left = left !== right ? 1 : 0; break;
      }
    }
    
    return left;
  }
  
  function parseFactor(): number {
    const token = peek();
    
    if (token.type === 'NUMBER') {
      consume();
      return token.value as number;
    }
    
    if (token.type === 'LPAREN') {
      consume(); // consume '('
      const result = parseExpression();
      if (peek().type === 'RPAREN') {
        consume(); // consume ')'
      }
      return result;
    }
    
    if (token.type === 'OPERATOR' && token.value === '-') {
      consume();
      return -parseFactor();
    }
    
    if (token.type === 'OPERATOR' && token.value === '+') {
      consume();
      return parseFactor();
    }
    
    // Default to 0 for unknown tokens
    return 0;
  }
  
  try {
    return parseExpression();
  } catch {
    return '#ERROR';
  }
}

// Parse function arguments
function parseArguments(argsString: string): (number | number[])[] {
  const args: (number | number[])[] = [];
  let current = '';
  let depth = 0;

  for (const char of argsString) {
    if (char === ',' && depth === 0) {
      args.push(parseArgumentValue(current.trim()));
      current = '';
    } else {
      if (char === '[') depth++;
      if (char === ']') depth--;
      current += char;
    }
  }

  if (current.trim()) {
    args.push(parseArgumentValue(current.trim()));
  }

  return args;
}

function parseArgumentValue(value: string): number | number[] {
  // Check if it's an array
  if (value.startsWith('[') && value.endsWith(']')) {
    const inner = value.slice(1, -1);
    if (!inner) return [];
    return inner.split(',').map(v => {
      const num = parseFloat(v.trim());
      return isNaN(num) ? 0 : num;
    });
  }
  
  // It's a single number
  const num = parseFloat(value);
  return isNaN(num) ? 0 : num;
}

// Get computed value (handles formulas)
export function getComputedValue(
  row: number,
  col: number,
  cells: Record<string, CellData>
): string {
  const key = getCellKey(row, col);
  const cellValue = cells[key]?.value;
  
  if (!cellValue) return '';
  if (!cellValue.startsWith('=')) return cellValue;
  
  const visitedCells = new Set<string>();
  visitedCells.add(key);
  return evaluateFormula(cellValue, cells, visitedCells);
}
