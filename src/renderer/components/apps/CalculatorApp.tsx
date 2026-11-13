import React, { useState, useEffect } from 'react';
import { Badge } from '../common/Badge';

export const CalculatorApp: React.FC = () => {
  const [display, setDisplay] = useState('0');
  const [prevVal, setPrevVal] = useState<number | null>(null);
  const [operation, setOperation] = useState<string | null>(null);
  const [waitingForOperand, setWaitingForOperand] = useState(false);
  const [isScientific, setIsScientific] = useState(false);

  const inputDigit = (digit: string) => {
    if (waitingForOperand) {
      setDisplay(digit);
      setWaitingForOperand(false);
    } else {
      setDisplay(display === '0' ? digit : display + digit);
    }
  };

  const inputDecimal = () => {
    if (waitingForOperand) {
      setDisplay('0.');
      setWaitingForOperand(false);
    } else if (!display.includes('.')) {
      setDisplay(display + '.');
    }
  };

  const clearAll = () => {
    setDisplay('0');
    setPrevVal(null);
    setOperation(null);
    setWaitingForOperand(false);
  };

  const performOperation = (nextOp: string) => {
    const inputValue = parseFloat(display);

    if (prevVal === null) {
      setPrevVal(inputValue);
    } else if (operation) {
      const currentValue = prevVal || 0;
      let result = currentValue;

      switch (operation) {
        case '+': result = currentValue + inputValue; break;
        case '-': result = currentValue - inputValue; break;
        case '×': result = currentValue * inputValue; break;
        case '÷': result = inputValue !== 0 ? currentValue / inputValue : 0; break;
      }

      setPrevVal(result);
      setDisplay(String(result));
    }

    setWaitingForOperand(true);
    setOperation(nextOp === '=' ? null : nextOp);
  };

  const scientificOp = (type: string) => {
    const val = parseFloat(display);
    let res = val;
    switch (type) {
      case 'sin': res = Math.sin(val); break;
      case 'cos': res = Math.cos(val); break;
      case 'tan': res = Math.tan(val); break;
      case 'sqrt': res = Math.sqrt(val); break;
      case 'sqr': res = val * val; break;
    }
    setDisplay(String(res));
    setWaitingForOperand(true);
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key >= '0' && e.key <= '9') inputDigit(e.key);
      else if (e.key === '.') inputDecimal();
      else if (e.key === '+') performOperation('+');
      else if (e.key === '-') performOperation('-');
      else if (e.key === '*') performOperation('×');
      else if (e.key === '/') performOperation('÷');
      else if (e.key === 'Enter' || e.key === '=') performOperation('=');
      else if (e.key === 'Escape') clearAll();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '16px', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
        <button
          onClick={() => setIsScientific(!isScientific)}
          style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-accent-primary)', cursor: 'pointer', fontWeight: 600 }}
        >
          {isScientific ? 'Standard Mode' : 'Scientific Mode'}
        </button>
        <Badge tier="simulated">In-App Engine</Badge>
      </div>

      {/* Screen Display */}
      <div
        style={{
          padding: '16px',
          borderRadius: 'var(--nova-radius-md)',
          backgroundColor: 'var(--nova-surface-base)',
          border: '1px solid var(--nova-border-subtle)',
          textAlign: 'right',
          fontSize: '28px',
          fontFamily: 'var(--nova-font-mono)',
          fontWeight: 700,
          marginBottom: '16px',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
        }}
      >
        {display}
      </div>

      {/* Keypad */}
      <div style={{ flex: 1, display: 'grid', gridTemplateColumns: isScientific ? 'repeat(5, 1fr)' : 'repeat(4, 1fr)', gap: '8px' }}>
        {isScientific && (
          <>
            <button onClick={() => scientificOp('sin')} style={btnStyle}>sin</button>
            <button onClick={() => scientificOp('cos')} style={btnStyle}>cos</button>
            <button onClick={() => scientificOp('tan')} style={btnStyle}>tan</button>
            <button onClick={() => scientificOp('sqrt')} style={btnStyle}>√</button>
            <button onClick={() => scientificOp('sqr')} style={btnStyle}>x²</button>
          </>
        )}

        <button onClick={clearAll} style={{ ...btnStyle, color: '#f43f5e' }}>AC</button>
        <button onClick={() => setDisplay(String(-parseFloat(display)))} style={btnStyle}>±</button>
        <button onClick={() => setDisplay(String(parseFloat(display) / 100))} style={btnStyle}>%</button>
        <button onClick={() => performOperation('÷')} style={{ ...btnStyle, backgroundColor: 'var(--nova-surface-raised)', color: 'var(--nova-accent-primary)' }}>÷</button>

        <button onClick={() => inputDigit('7')} style={btnStyle}>7</button>
        <button onClick={() => inputDigit('8')} style={btnStyle}>8</button>
        <button onClick={() => inputDigit('9')} style={btnStyle}>9</button>
        <button onClick={() => performOperation('×')} style={{ ...btnStyle, backgroundColor: 'var(--nova-surface-raised)', color: 'var(--nova-accent-primary)' }}>×</button>

        <button onClick={() => inputDigit('4')} style={btnStyle}>4</button>
        <button onClick={() => inputDigit('5')} style={btnStyle}>5</button>
        <button onClick={() => inputDigit('6')} style={btnStyle}>6</button>
        <button onClick={() => performOperation('-')} style={{ ...btnStyle, backgroundColor: 'var(--nova-surface-raised)', color: 'var(--nova-accent-primary)' }}>-</button>

        <button onClick={() => inputDigit('1')} style={btnStyle}>1</button>
        <button onClick={() => inputDigit('2')} style={btnStyle}>2</button>
        <button onClick={() => inputDigit('3')} style={btnStyle}>3</button>
        <button onClick={() => performOperation('+')} style={{ ...btnStyle, backgroundColor: 'var(--nova-surface-raised)', color: 'var(--nova-accent-primary)' }}>+</button>

        <button onClick={() => inputDigit('0')} style={{ ...btnStyle, gridColumn: 'span 2' }}>0</button>
        <button onClick={inputDecimal} style={btnStyle}>.</button>
        <button onClick={() => performOperation('=')} style={{ ...btnStyle, backgroundColor: 'var(--nova-accent-primary)', color: 'var(--nova-text-inverse)', fontWeight: 700 }}>=</button>
      </div>
    </div>
  );
};

const btnStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  borderRadius: 'var(--nova-radius-md)',
  backgroundColor: 'var(--nova-surface-base)',
  border: '1px solid var(--nova-border-subtle)',
  fontSize: 'var(--nova-text-md)',
  fontWeight: 500,
  cursor: 'pointer',
  transition: 'background-color var(--nova-duration-fast) var(--nova-ease-standard)',
};

