import styled from '@emotion/styled';

export const TextInput = styled.input`
  display: block;
  background-color: #fff;
  border: 1px solid ${(props) => props.theme.colors.borderDark};
  border-radius: 3px;
  color: ${(props) => props.theme.colors.text};
  padding: 9px 10px 8px;
  font-size: 14px;
  font-weight: normal;
  box-shadow: none;
  width: 100%;
  transition: border-color ease-in-out 0.15s;
  appearance: none;

  &:focus {
    border-color: ${(props) => props.theme.colors.primary};
  }
`;
