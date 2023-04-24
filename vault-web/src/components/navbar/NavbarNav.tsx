import styled from '@emotion/styled';

export const NavbarNav = styled.nav`
  display: flex;
  flex-direction: row;
  padding-top: ${({ theme }) => (theme.isMobile ? 0 : '11px')};
  margin-right: ${({ theme }) => (theme.isMobile ? 0 : '-6px')};
`;
